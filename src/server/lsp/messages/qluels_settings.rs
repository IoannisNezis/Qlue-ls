use serde::{Deserialize, Serialize};

use crate::server::{
    configuration::Settings,
    lsp::{
        rpc::{NotificationMessageBase, RequestMessageBase, ResponseMessageBase},
        LspMessage, NotificationMarker, RequestMarker, ResponseMarker,
    },
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct DefaultSettingsRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl LspMessage for DefaultSettingsRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/defaultSettings")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        Some(&self.base.id)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DefaultSettingsResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    pub result: DefaultSettingsResult,
}

impl LspMessage for DefaultSettingsResponse {
    type Kind = ResponseMarker;

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        self.base.request_id()
    }

    fn method(&self) -> Option<&str> {
        None
    }
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

impl LspMessage for ChangeSettingsNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/changeSettings")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}
pub type ChangeSettingsParams = Settings;
