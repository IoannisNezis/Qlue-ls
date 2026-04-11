use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    Server,
    configuration::Settings,
    lsp::{
        ChangeSettingsNotification, DefaultSettingsRequest, DefaultSettingsResponse,
        errors::LSPError,
    },
};

#[tracing::instrument(skip_all, fields(id = %request.base.id))]
pub(super) async fn handle_default_settings_request(
    server_rc: Rc<Mutex<Server>>,
    request: DefaultSettingsRequest,
) -> Result<(), LSPError> {
    server_rc
        .lock()
        .await
        .send_message(DefaultSettingsResponse::new(
            request.base.id,
            Settings::default(),
        ))
}

#[tracing::instrument(skip_all)]
pub(super) async fn handle_change_settings_notification(
    server_rc: Rc<Mutex<Server>>,
    request: ChangeSettingsNotification,
) -> Result<(), LSPError> {
    // TODO: Merge settings instead of replaceing everything
    server_rc.lock().await.settings = request.params;
    Ok(())
}
