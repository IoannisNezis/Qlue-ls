use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    lsp::{errors::LSPError, WorkspaceEditResponse},
    Server,
};

pub(super) async fn handle_workspace_edit_response(
    _server_rc: Rc<Mutex<Server>>,
    response: WorkspaceEditResponse,
) -> Result<(), LSPError> {
    if let Some(result) = response.result {
        if !result.applied {
            log::warn!("Work space edit did not get applied");
        }
    }
    Ok(())
}
