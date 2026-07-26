use serde::{Deserialize, Serialize};

use crate::server::{
    configuration::Settings,
    lsp::{
        LspMessage,
        rpc::{NotificationMessageBase, RequestMessageBase, ResponseMessageBase},
    },
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct DefaultSettingsRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl LspMessage for DefaultSettingsRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct DefaultSettingsResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    pub result: DefaultSettingsResult,
}

impl LspMessage for DefaultSettingsResponse {}

impl DefaultSettingsResponse {
    pub(crate) fn new(
        id: crate::server::lsp::rpc::RequestId,
        settings: DefaultSettingsResult,
    ) -> Self {
        Self {
            base: ResponseMessageBase::success(&id),
            result: settings,
        }
    }
}

pub type DefaultSettingsResult = Settings;

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChangeSettingsNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: ChangeSettingsParams,
}

impl LspMessage for ChangeSettingsNotification {}
/// The payload of `qlueLs/changeSettings`.
///
/// NOTE: kept as raw JSON on purpose. The notification carries a *partial*
/// settings object that is merged into the current settings, so the handler has
/// to distinguish "key absent" from "key set to its default value", which a
/// deserialized [`Settings`] can no longer express.
/// See `message_handler::settings::handle_change_settings_notification`.
pub type ChangeSettingsParams = serde_json::Value;
