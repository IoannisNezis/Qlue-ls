use serde::{Deserialize, Serialize};

use crate::server::{
    configuration::Settings,
    lsp::rpc::{NotificationMessageBase, ResponseMessageBase},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct DefaultSettingsResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    pub result: DefaultSettingsResult,
}

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
pub type ChangeSettingsParams = Settings;
