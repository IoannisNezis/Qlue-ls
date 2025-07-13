use std::{collections::HashMap, fmt::Debug, rc::Rc};

use futures::lock::Mutex;
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::server::lsp::LspMessage;

#[derive(Serialize)]
pub(super) struct TraceEvent {
    pub(super) name: String,
    pub(super) id: String,
    pub(super) cat: String,
    pub(super) ph: String,
    pub(super) ts: u64,
    pub(super) pid: String,
    pub(super) tid: String,
    pub(super) args: Option<serde_json::Value>,
}

#[derive(Default)]
pub(super) struct TraceFile {
    pub(super) trace_events: Vec<TraceEvent>,
    method_map: HashMap<String, String>,
}
impl TraceFile {
    pub(crate) fn add(&mut self, mut trace_event: TraceEvent) {
        if &trace_event.ph == "B" {
            self.add_method_record(&trace_event.id, &trace_event.name);
        } else if &trace_event.ph == "E" {
            let method = self.pop_method_record(&trace_event.id);
            trace_event.name = method;
        }

        self.trace_events.push(trace_event);
    }

    pub(crate) fn add_method_record(&mut self, id: &str, method: &str) {
        self.method_map.insert(id.to_string(), method.to_string());
    }

    pub(crate) fn pop_method_record(&mut self, id: &str) -> String {
        self.method_map.remove(id).unwrap()
    }

    pub(crate) fn dump(&self) {
        let json_string = self
            .trace_events
            .iter()
            .map(|event| serde_json::to_string(event).unwrap())
            .collect::<Vec<_>>()
            .join(",");
        log::debug!(r#"{{"traceEvents":[{json_string}],"displayTimeUnit":"ms"}}"#);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("now should be after 1970")
        .as_micros() as u64
}

#[cfg(target_arch = "wasm32")]
pub(super) fn now_us() -> u64 {
    use wasm_bindgen::JsCast;

    (js_sys::global()
        .unchecked_into::<web_sys::WorkerGlobalScope>()
        .performance()
        .expect("performance should be available")
        .now()
        * 1000.0) as u64
}

pub(super) fn log_trace<T>(trace_file: Rc<Mutex<TraceFile>>, message: &T)
where
    T: LspMessage + Debug,
{
    log::debug!("{:?}:{:?}", message.method(), message.kind());
    if let Some(trace_event) = match (message.method(), message.id()) {
        (Some(method), Some(id)) => Some(TraceEvent {
            name: method.to_string(),
            id: id.to_string(),
            cat: "lsp".to_string(),
            ph: "B".to_string(),
            ts: now_us(),
            pid: "qlue-ls".to_string(),
            tid: "Main thread".to_string(),
            args: None,
        }),
        (None, Some(id)) => Some(TraceEvent {
            name: id.to_string(),
            id: id.to_string(),
            cat: "lsp".to_string(),
            ph: "E".to_string(),
            ts: now_us(),
            pid: "qlue-ls".to_string(),
            tid: "Main thread".to_string(),
            args: None,
        }),
        (Some(_method), None) => None,
        (None, None) => {
            panic!(
                "A message without a method or id should not exist:\n {:?}",
                message
            )
        }
    } {
        spawn_local(async move {
            let mut trace_file = trace_file.lock().await;
            trace_file.add(trace_event);
        });
    }
}
