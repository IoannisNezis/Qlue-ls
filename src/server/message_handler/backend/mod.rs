use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    fetch::check_server_availability,
    lsp::{
        errors::{ErrorCode, LSPError},
        AddBackendNotification, PingBackendRequest, PingBackendResponse,
        UpdateDefaultBackendNotification,
    },
    Server,
};

pub(super) async fn handle_update_backend_default_notification(
    server_rc: Rc<Mutex<Server>>,
    notification: UpdateDefaultBackendNotification,
) -> Result<(), LSPError> {
    log::info!("new default backend: {}", notification.params.backend_name);
    let mut server = server_rc.lock().await;
    if server
        .state
        .get_backend(&notification.params.backend_name)
        .is_none()
    {
        return Err(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("Unknown backend \"{}\"", notification.params.backend_name),
        ));
    }
    server
        .state
        .set_default_backend(notification.params.backend_name);
    Ok(())
}

pub(super) async fn handle_ping_backend_request(
    server_rc: Rc<Mutex<Server>>,
    request: PingBackendRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let backend = match request.params.backend_name {
        Some(ref name) => server.state.get_backend(name).ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("got ping request for unknown backend: \"{}\"", name),
        )),
        None => server.state.get_default_backend().ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            "no backend or default backend provided",
        )),
    }?;
    let health_check_url = &backend.health_check_url.as_ref().unwrap_or(&backend.url);
    let availible = check_server_availability(health_check_url).await;
    server.send_message(PingBackendResponse::new(request.get_id(), availible))
}

pub(super) async fn handle_add_backend_notification(
    server_rc: Rc<Mutex<Server>>,
    request: AddBackendNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    server.state.add_backend(request.params.backend.clone());
    if request.params.default {
        server
            .state
            .set_default_backend(request.params.backend.name.clone());
    }
    if let Some(prefix_map) = request.params.prefix_map {
        server
            .state
            .add_prefix_map(request.params.backend.name.clone(), prefix_map)
            .await
            .map_err(|err| {
                log::error!("{}", err);
                LSPError::new(
                    ErrorCode::InvalidParams,
                    &format!("Could not load prefix map:\n\"{}\"", err),
                )
            })?;
    };
    if let Some(completion_querys) = request.params.queries {
        for (query_name, query) in completion_querys.iter() {
            server
                .tools
                .tera
                .add_raw_template(
                    &format!("{}-{}", request.params.backend.name, query_name),
                    query,
                )
                .map_err(|err| {
                    log::error!("{}", err);
                    LSPError::new(
                        ErrorCode::InvalidParams,
                        &format!("Could load template:\n\"{query_name}\"\n{}", err),
                    )
                })?;
        }
    }
    Ok(())
}
