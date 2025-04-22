use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    lsp::{errors::LSPError, SetTraceNotification},
    Server,
};

pub(super) async fn handle_set_trace_notifcation(
    server: Rc<Mutex<Server>>,
    set_trace_notification: SetTraceNotification,
) -> Result<(), LSPError> {
    log::info!("Trace set to: {:?}", set_trace_notification.params.value);
    server.lock().await.state.trace_value = set_trace_notification.params.value;
    Ok(())
}
