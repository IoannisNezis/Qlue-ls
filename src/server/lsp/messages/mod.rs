mod command;
pub mod diagnostic;
mod initialize;
mod progress;
mod qluels_addbackend;
mod qluels_jump;
mod qluels_operationtype;
mod qluels_pingbackend;
mod qluels_settings;
mod qluels_updatebackenddefault;
mod shutdown;
mod textdocument_codeaction;
mod textdocument_completion;
mod textdocument_diagnostic;
mod textdocument_didchange;
mod textdocument_didopen;
mod textdocument_didsave;
mod textdocument_formatting;
mod textdocument_hover;
mod textdocument_publishdiagnostics;
mod trace;
mod utils;
mod window_showmessage;
mod workspace;
mod workspace_applyedit;

pub use command::*;
pub use initialize::*;
pub use progress::*;
pub use qluels_addbackend::*;
pub use qluels_jump::*;
pub use qluels_operationtype::*;
pub use qluels_pingbackend::*;
pub use qluels_settings::*;
pub use qluels_updatebackenddefault::*;
pub use shutdown::*;
pub use textdocument_codeaction::*;
pub use textdocument_completion::*;
pub use textdocument_diagnostic::*;
pub use textdocument_didchange::*;
pub use textdocument_didopen::*;
pub use textdocument_didsave::*;
pub use textdocument_formatting::*;
pub use textdocument_hover::*;
pub use trace::*;
pub use workspace::*;
pub use workspace_applyedit::*;

use crate::server::lsp::rpc::RequestId;

#[derive(Debug)]
pub enum LspMessageKind {
    Request,
    Response,
    Notification,
}

pub enum RequestMarker {}
pub enum ResponseMarker {}
pub enum NotificationMarker {}

pub trait LspMessageKindMarker {
    fn kind() -> LspMessageKind;
}

impl LspMessageKindMarker for RequestMarker {
    fn kind() -> LspMessageKind {
        LspMessageKind::Request
    }
}

impl LspMessageKindMarker for ResponseMarker {
    fn kind() -> LspMessageKind {
        LspMessageKind::Response
    }
}

impl LspMessageKindMarker for NotificationMarker {
    fn kind() -> LspMessageKind {
        LspMessageKind::Notification
    }
}

pub trait LspMessage {
    type Kind: LspMessageKindMarker;
    fn kind(&self) -> LspMessageKind {
        <RequestMarker as LspMessageKindMarker>::kind()
    }
    fn method(&self) -> Option<&str>;
    fn id(&self) -> Option<&RequestId>;
}
