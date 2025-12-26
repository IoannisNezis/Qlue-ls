use futures::lock::Mutex;
use std::rc::Rc;

use crate::server::{
    Server,
    lsp::{CancelQueryNotification, errors::LSPError},
};

pub(super) async fn handle_cancel_notification(
    server_rc: Rc<Mutex<Server>>,
    notification: CancelQueryNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    if let Some(abort_fn) = server
        .state
        .get_running_request(&notification.params.query_id)
    {
        abort_fn();
    } else {
        log::error!(
            "Received cancel notification for unknown query: {}",
            notification.params.query_id
        );
    }
    Ok(())
}
