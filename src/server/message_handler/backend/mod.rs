use crate::server::{
    fetch::check_server_availability,
    lsp::{
        errors::{ErrorCode, LSPError},
        PingBackendRequest, PingBackendResponse, SetBackendNotification,
    },
    Server,
};

pub(super) async fn handle_ping_backend_request(
    server: &mut Server,
    request: PingBackendRequest,
) -> Result<(), LSPError> {
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
    server: &mut Server,
    request: SetBackendNotification,
) -> Result<(), LSPError> {
    server.state.add_backend(request.params.backend.clone());
    if request.params.default {
        server
            .state
            .set_default_backend(request.params.backend.name.clone());
    }
    if let Some(prefix_map) = request.params.prefix_map {
        server
            .state
            .add_prefix_map(request.params.backend.name, prefix_map)
            .await
            .map_err(|err| {
                log::error!("{}", err);
                LSPError::new(
                    ErrorCode::InvalidParams,
                    &format!("Could not load prefix map:\n\"{}\"", err),
                )
            })?;
    };
    Ok(())
}
