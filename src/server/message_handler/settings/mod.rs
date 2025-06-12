use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    configuration::Settings,
    lsp::{
        errors::LSPError, rpc::RequestMessage, ChangeSettingsNotification, DefaultSettingsResponse,
    },
    Server,
};

pub(super) async fn handle_default_settings_request(
    server_rc: Rc<Mutex<Server>>,
    request: RequestMessage,
) -> Result<(), LSPError> {
    server_rc
        .lock()
        .await
        .send_message(DefaultSettingsResponse::new(
            request.id,
            Settings::default(),
        ))
}

pub(super) async fn handle_change_settings_notification(
    server_rc: Rc<Mutex<Server>>,
    request: ChangeSettingsNotification,
) -> Result<(), LSPError> {
    server_rc.lock().await.settings = request.params;
    log::info!("Changed settings");
    Ok(())
}
