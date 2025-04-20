use std::{cell::RefCell, rc::Rc};

use crate::server::{
    lsp::{errors::LSPError, SetTraceNotification},
    Server,
};

pub(super) async fn handle_set_trace_notifcation(
    server: Rc<RefCell<Server>>,
    set_trace_notification: SetTraceNotification,
) -> Result<(), LSPError> {
    log::info!("Trace set to: {:?}", set_trace_notification.params.value);
    server.borrow_mut().state.trace_value = set_trace_notification.params.value;
    Ok(())
}
