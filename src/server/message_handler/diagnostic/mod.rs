mod undeclared_prefix;
mod unused_prefix;
use crate::server::{
    anaysis::get_all_uncompacted_uris,
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        errors::ResponseError,
        textdocument::TextDocumentItem,
        DiagnosticRequest, DiagnosticResponse,
    },
    Server,
};

pub fn handle_diagnostic_request(
    server: &mut Server,
    request: DiagnosticRequest,
) -> Result<DiagnosticResponse, ResponseError> {
    Ok(DiagnosticResponse::new(
        request.get_id(),
        collect_diagnostics(server, &request.params.text_document.uri)?,
    ))
}

pub fn collect_diagnostics(
    server: &Server,
    document_uri: &str,
) -> Result<Vec<Diagnostic>, ResponseError> {
    let document = server.state.get_document(document_uri)?;
    let mut diagnostics = Vec::new();
    if let Some(unused_prefix_diagnostics) = unused_prefix::diagnostics(document) {
        diagnostics.extend(unused_prefix_diagnostics);
    }
    if let Some(undeclared_prefix_diagnostics) = undeclared_prefix::diagnostics(document) {
        diagnostics.extend(undeclared_prefix_diagnostics);
    }
    diagnostics.extend(uncompacted_uris(server, document)?);
    Ok(diagnostics)
}

// fn undefined_select_binding(
//     server: &Server,
//     document: &TextDocumentItem,
// ) -> Result<impl Iterator<Item = Diagnostic>, ResponseError> {
//     todo!()
// }

fn uncompacted_uris<'a>(
    server: &'a Server,
    document: &TextDocumentItem,
) -> Result<impl Iterator<Item = Diagnostic> + use<'a>, ResponseError> {
    let uncompacted_uris = get_all_uncompacted_uris(server, &document.uri)?;
    let diagnostics = uncompacted_uris.into_iter().filter_map(|(uri, range)| {
        match server.shorten_uri(&uri[1..uri.len() - 1]) {
            Some((prefix, namespace, curie)) => Some(Diagnostic {
                source: Some("qlue-ls".to_string()),
                code: Some(DiagnosticCode::String("uncompacted-uri".to_string())),
                range,
                severity: DiagnosticSeverity::Information,
                message: format!("You might want to shorten this Uri\n{} -> {}", uri, curie),
                data: Some(LSPAny::LSPArray(vec![
                    LSPAny::String(prefix),
                    LSPAny::String(namespace),
                    LSPAny::String(curie),
                ])),
            }),
            None => None,
        }
    });
    Ok(diagnostics)
}
