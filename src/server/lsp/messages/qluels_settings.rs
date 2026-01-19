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
pub type ChangeSettingsParams = Settings;
