use crate::server::{
    fetch::check_server_availability,
    lsp::{errors::LSPError, SetBackendRequest, SetBackendResponse},
    Server,
};

pub(super) async fn handle_set_backend_request(
    server: &mut Server,
    request: SetBackendRequest,
) -> Result<(), LSPError> {
    log::info!(
        r#"Set backend "{}": <{}>"#,
        request.params.name,
        request.params.url
    );
    let health_check_url = &request
        .params
        .health_check_url
        .as_ref()
        .unwrap_or(&request.params.url);
    log::info!("Testing availibilty of <{}>", health_check_url);
    let availible = check_server_availability(health_check_url).await;
    log::info!(
        "Servive: {}",
        match availible {
            true => "availible",
            false => "unavailible",
        }
    );
    let id = request.get_id().clone();
    server.state.set_backend(request.params);
    server.send_message(SetBackendResponse::new(&id, availible))?;
    Ok(())
}
