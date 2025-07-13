use std::{process::exit, rc::Rc};

use futures::lock::Mutex;

use crate::server::{
    configuration::BackendConfiguration,
    lsp::{
        errors::{ErrorCode, LSPError},
        ExitNotification, InitializeRequest, InitializeResponse, InitializedNotification,
        ProgressNotification, ShutdownRequest, ShutdownResponse,
    },
    state::ServerStatus,
    Server,
};

pub(super) async fn handle_shutdown_request(
    server_rc: Rc<Mutex<Server>>,
    request: ShutdownRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    log::info!("Received shutdown request, preparing to shut down");
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
            server.send_message(ShutdownResponse::new(&request.base.id))
        }
    }
}

pub(super) async fn handle_initialize_request(
    server_rc: Rc<Mutex<Server>>,
    initialize_request: InitializeRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
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
            server.client_capabilities = Some(initialize_request.params.capabilities.clone());
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

                let mut backend_configs = Vec::new();
                if let Some(config) = server.settings.backends.as_ref() {
                    for backend_config in config.backends.iter().map(|x| x.1).cloned() {
                        backend_configs.push(backend_config)
                    }
                }
                for config in backend_configs.into_iter() {
                    let BackendConfiguration {
                        backend,
                        request_method,
                        prefix_map,
                        default,
                        queries,
                    } = config;

                    server
                        .state
                        .add_prefix_map(backend.name.clone(), prefix_map)
                        .await
                        .map_err(|err| {
                            log::error!("{}", err);
                            LSPError::new(
                                ErrorCode::InvalidParams,
                                &format!("Could not load prefix map:\n\"{}\"", err),
                            )
                        })?;
                    if let Some(method) = request_method {
                        server
                            .state
                            .add_backend_request_method(&backend.name, method);
                    };
                    server
                        .tools
                        .tera
                        .add_raw_template(
                            &format!("{}-subjectCompletion", &backend.name),
                            &queries.subject_completion,
                        )
                        .map_err(|err| {
                            log::error!("{}", err);
                            LSPError::new(
                                ErrorCode::InvalidParams,
                                &format!(
                                    "Could not load template: subjectCompletion of backend {}",
                                    &backend.name
                                ),
                            )
                        })?;
                    server
                        .tools
                        .tera
                        .add_raw_template(
                            &format!("{}-predicateCompletion", &backend.name),
                            &queries.predicate_completion,
                        )
                        .map_err(|err| {
                            log::error!("{}", err);
                            LSPError::new(
                                ErrorCode::InvalidParams,
                                &format!(
                                    "Could not load template: predicateCompletion of backend {}",
                                    &backend.name
                                ),
                            )
                        })?;
                    server
                        .tools
                        .tera
                        .add_raw_template(
                            &format!("{}-objectCompletion", &backend.name),
                            &queries.object_completion,
                        )
                        .map_err(|err| {
                            log::error!("{}", err);
                            LSPError::new(
                                ErrorCode::InvalidParams,
                                &format!(
                                    "Could not load template: objectCompletion of backend {}",
                                    &backend.name
                                ),
                            )
                        })?;
                    if default {
                        server.state.set_default_backend(backend.name.clone());
                    }

                    server.state.add_backend(backend);
                }

                let progress_report_1 = ProgressNotification::report_notification(
                    work_done_token.clone(),
                    Some(false),
                    Some("testing availability of endpoint"),
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

pub(super) async fn handle_initialized_notification(
    server_rc: Rc<Mutex<Server>>,
    _initialized_notification: InitializedNotification,
) -> Result<(), LSPError> {
    log::info!("initialization completed");
    server_rc.lock().await.state.status = ServerStatus::Running;
    Ok(())
}

pub(super) async fn handle_exit_notification(
    _server_rc: Rc<Mutex<Server>>,
    _initialized_notification: ExitNotification,
) -> Result<(), LSPError> {
    log::info!("Received exit notification, shutting down!");
    exit(0);
}
