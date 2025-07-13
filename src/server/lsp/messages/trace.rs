use serde::{Deserialize, Serialize};

use crate::server::lsp::{rpc::NotificationMessageBase, LspMessage, NotificationMarker};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTraceNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: SetTraceParams,
}

impl LspMessage for SetTraceNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("$/setTrace")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTraceParams {
    pub value: TraceValue,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TraceValue {
    Off,
    Message,
    Verbose,
}

#[cfg(test)]
mod test {

    use crate::server::lsp::rpc::NotificationMessageBase;

    use super::{SetTraceNotification, TraceValue};

    #[test]
    fn serialize() {
        let set_trace_notification = SetTraceNotification {
            base: NotificationMessageBase::new("$/setTrace"),
            params: super::SetTraceParams {
                value: TraceValue::Off,
            },
        };
        assert_eq!(
            serde_json::to_string(&set_trace_notification).unwrap(),
            r#"{"jsonrpc":"2.0","method":"$/setTrace","params":{"value":"off"}}"#
        )
    }
}
