use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    Server,
    lsp::{
        AddBackendNotification, GetBackendRequest, GetBackendResponse, ListBackendsRequest,
        ListBackendsResponse, PingBackendRequest, PingBackendResponse,
        UpdateDefaultBackendNotification,
        errors::{ErrorCode, LSPError},
    },
    sparql_operations::check_server_availability,
};

pub(super) async fn handle_update_backend_default_notification(
    server_rc: Rc<Mutex<Server>>,
    notification: UpdateDefaultBackendNotification,
) -> Result<(), LSPError> {
    log::info!("new default backend: {}", notification.params.backend_name);
    // TODO: update default fild in backends state
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
    let backend = {
        let server = server_rc.lock().await;
        match request.params.backend_name {
            Some(ref name) => server.state.get_backend(name).cloned().ok_or(LSPError::new(
                ErrorCode::InvalidParams,
                &format!("got ping request for unknown backend: \"{}\"", name),
            )),
            None => server
                .state
                .get_default_backend()
                .cloned()
                .ok_or(LSPError::new(
                    ErrorCode::InvalidParams,
                    "no backend or default backend provided",
                )),
        }?
    };
    let health_check_url = &backend.health_check_url.as_ref().unwrap_or(&backend.url);
    let available = check_server_availability(health_check_url).await;
    server_rc
        .lock()
        .await
        .send_message(PingBackendResponse::new(request.get_id(), available))
}

pub(super) async fn handle_add_backend_notification(
    server_rc: Rc<Mutex<Server>>,
    request: AddBackendNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;

    server
        .state
        .load_prefix_map(request.params.name.clone(), &request.params.prefix_map)?;
    server.load_templates(&request.params.name, request.params.queries.clone())?;
    let backend_name = request.params.name.clone();
    let default = request.params.default;
    server.state.add_backend(request.params);
    if default {
        server.state.set_default_backend(backend_name);
    }
    Ok(())
}

pub(super) async fn handle_get_backend_request(
    server_rc: Rc<Mutex<Server>>,
    request: GetBackendRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let (backend, error_message) = match request.params.backend {
        Some(ref name) => (
            server.state.get_backend(name),
            format!("Backend \"{}\" not found.", name),
        ),
        None => (
            server.state.get_default_backend(),
            "No default backend is configured.".to_string(),
        ),
    };
    server.send_message(GetBackendResponse::new(
        request.get_id(),
        backend,
        &error_message,
    ))
}

pub(super) async fn handle_list_backends_request(
    server_rc: Rc<Mutex<Server>>,
    request: ListBackendsRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    server.send_message(ListBackendsResponse::new(
        request.get_id(),
        server.state.get_all_backends(),
    ))
}
