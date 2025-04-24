use std::{process::exit, rc::Rc};

use futures::lock::Mutex;

use crate::server::{
    lsp::{
        errors::{ErrorCode, LSPError},
        rpc::{NotificationMessage, RequestMessage},
        InitializeRequest, InitializeResponse, ProgressNotification, ShutdownResponse,
    },
    state::ServerStatus,
    Server,
};

pub(super) async fn handle_shutdown_request(
    server_rc: Rc<Mutex<Server>>,
    request: RequestMessage,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    log::info!("Recieved shutdown request, preparing to shut down");
    match server.state.status {
        ServerStatus::Initializing => Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "The Server is not yet initialized",
        )),
        ServerStatus::ShuttingDown => Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "The Server is already shutting down",
        )),
        ServerStatus::Running => {
            server.state.status = ServerStatus::ShuttingDown;
            server.send_message(ShutdownResponse::new(&request.id))
        }
    }
}

pub(super) async fn handle_initialize_request(
    server_rc: Rc<Mutex<Server>>,
    initialize_request: InitializeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    match server.state.status {
        ServerStatus::Initializing => {
            if let Some(ref client_info) = initialize_request.params.client_info {
                log::info!(
                    "Connected to: {} {}",
                    client_info.name,
                    client_info
                        .version
                        .clone()
                        .unwrap_or("no version specified".to_string())
                );
            }
            if let Some(ref work_done_token) =
                initialize_request.params.progress_params.work_done_token
            {
                let init_progress_begin_notification = ProgressNotification::begin_notification(
                    work_done_token.clone(),
                    &format!("setup qlue-ls v{}", server.get_version()),
                    Some(false),
                    Some("init"),
                    Some(0),
                );
                server.send_message(init_progress_begin_notification)?;

                let progress_report_1 = ProgressNotification::report_notification(
                    work_done_token.clone(),
                    Some(false),
                    Some("testing availibility of endpoint"),
                    Some(30),
                );
                server.send_message(progress_report_1)?;

                let progress_report_2 = ProgressNotification::report_notification(
                    work_done_token.clone(),
                    Some(false),
                    Some("request prefixes from endpoint"),
                    Some(60),
                );
                server.send_message(progress_report_2)?;

                let init_progress_end_notification = ProgressNotification::end_notification(
                    work_done_token.clone(),
                    Some("qlue-ls initialized"),
                );

                server.send_message(init_progress_end_notification)?;
            }
            server.send_message(InitializeResponse::new(
                initialize_request.get_id(),
                &server.capabilities,
                &server.server_info,
            ))
        }
        _ => Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "The Server is already initialized",
        )),
    }
}

pub(super) async fn handle_initialized_notifcation(
    server_rc: Rc<Mutex<Server>>,
    _initialized_notification: NotificationMessage,
) -> Result<(), LSPError> {
    log::info!("initialization completed");
    server_rc.lock().await.state.status = ServerStatus::Running;
    Ok(())
}

pub(super) async fn handle_exit_notifcation(
    _server_rc: Rc<Mutex<Server>>,
    _initialized_notification: NotificationMessage,
) -> Result<(), LSPError> {
    log::info!("Recieved exit notification, shutting down!");
    exit(0);
}
