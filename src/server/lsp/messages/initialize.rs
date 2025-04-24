use crate::server::lsp::{
    capabilities::ServerCapabilities,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    workdoneprogress::WorkDoneProgressParams,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct InitializeRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: InitializeParams,
}

impl InitializeRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub process_id: ProcessId,
    pub client_info: Option<ClientInfo>,
    #[serde(flatten)]
    pub progress_params: WorkDoneProgressParams,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ProcessId {
    Integer(i32),
    Null,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct ServerInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct InitializeResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: InitializeResult,
}

impl InitializeResponse {
    pub fn new(
        id: &RequestId,
        capabilities: &ServerCapabilities,
        server_info: &ServerInfo,
    ) -> Self {
        InitializeResponse {
            base: ResponseMessageBase::success(id),
            result: InitializeResult {
                capabilities: capabilities.clone(),
                server_info: Some(server_info.clone()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        rpc::{Message, RequestId, RequestMessageBase},
        workdoneprogress::{ProgressToken, WorkDoneProgressParams},
        ClientInfo, ProcessId,
    };

    use super::{InitializeParams, InitializeRequest};

    #[test]
    fn deserialize() {
        let message = br#"{"jsonrpc":"2.0","id": 1,"method":"initialize","params":{"processId":null,"clientInfo":{"name":"dings","version":"42.1"},"workDoneToken":"1"}}"#;
        let init_request: InitializeRequest = serde_json::from_slice(message).unwrap();
        assert_eq!(
            init_request,
            InitializeRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "initialize".to_string(),
                    id: RequestId::Integer(1),
                },
                params: InitializeParams {
                    process_id: ProcessId::Null,
                    client_info: Some(ClientInfo {
                        name: "dings".to_string(),
                        version: Some("42.1".to_string())
                    }),
                    progress_params: WorkDoneProgressParams {
                        work_done_token: Some(ProgressToken::Text("1".to_string()))
                    }
                }
            }
        );
    }
}
