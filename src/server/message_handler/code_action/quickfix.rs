use super::Diagnostic;
use crate::server::{
    anaysis::namespace_is_declared,
    common::{serde_parse, UncompactedUrisDiagnosticData},
    lsp::{
        base_types::LSPAny,
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextEdit},
        CodeAction, CodeActionKind, WorkspaceEdit,
    },
    message_handler::diagnostic,
    Server,
};
use log::error;
use std::collections::HashMap;

pub(super) fn get_quickfix(
    server: &mut Server,
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, LSPError> {
    if let Some(code) = diagnostic.code.as_ref() {
        if code == &*diagnostic::undeclared_prefix::CODE {
            declare_prefix(server, document_uri, diagnostic)
        } else if code == &*diagnostic::uncompacted_uri::CODE {
            shorten_uri(server, document_uri, diagnostic)
        } else if code == &*diagnostic::unused_prefix::CODE {
            remove_prefix_declaration(document_uri, diagnostic)
        } else {
            log::warn!("Unknown diagnostic code: {:?}", code);
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn remove_prefix_declaration(
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, LSPError> {
    let mut code_action =
        CodeAction::new("remove prefix declaration", Some(CodeActionKind::QuickFix));
    code_action.add_edit(document_uri, TextEdit::new(diagnostic.range, ""));
    Ok(Some(code_action))
}

fn shorten_uri(
    server: &mut Server,
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, LSPError> {
    match diagnostic.data {
        Some(data) => {
            let UncompactedUrisDiagnosticData(prefix, namespace, curie): UncompactedUrisDiagnosticData =
                serde_parse(data)?;
            let mut code_action = CodeAction::new("Shorten URI", Some(CodeActionKind::QuickFix));
            code_action.add_edit(document_uri, TextEdit::new(diagnostic.range, &curie));
            if !namespace_is_declared(&mut server.state, document_uri, &prefix)? {
                code_action.add_edit(
                    document_uri,
                    TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, namespace),
                    ),
                );
            }
            Ok(Some(code_action))
        }
        None => {
            error!("Data-field is missing in \"uncompacted-uri\" diagnostic");
            Err(LSPError::new(
                ErrorCode::InvalidRequest,
                "Data-field is missing in \"uncompacted-uri\" diagnostic",
            ))
        }
    }
}

fn declare_prefix(
    server: &Server,
    document_uri: &str,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, LSPError> {
    if let Some(LSPAny::String(prefix)) = &diagnostic.data {
        if let Some(Ok(record)) = server
            .state
            .get_default_converter()
            .map(|converter| converter.find_by_prefix(prefix))
        {
            Ok(Some(CodeAction {
                title: format!("Declare prefix \"{}\"", prefix),
                kind: Some(CodeActionKind::QuickFix),
                edit: WorkspaceEdit {
                    changes: HashMap::from([(
                        document_uri.to_string(),
                        vec![TextEdit::new(
                            Range::new(0, 0, 0, 0),
                            &format!("PREFIX {}: <{}>\n", prefix, record.uri_prefix),
                        )],
                    )]),
                },
                diagnostics: vec![diagnostic],
            }))
        } else {
            Ok(None)
        }
    } else {
        Err(LSPError::new(
            ErrorCode::InvalidParams,
            "expected prefix in undeclared-prefix data... was disapointed",
        ))
    }
}

#[cfg(test)]
mod test {

    use crate::server::{
        lsp::{
            base_types::LSPAny,
            diagnostic::{self, Diagnostic},
            textdocument::{Range, TextDocumentItem, TextEdit},
        },
        message_handler::code_action::quickfix::shorten_uri,
        state::ServerState,
        Server,
    };
    use indoc::indoc;

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let document = TextDocumentItem::new("uri", text);
        state.add_document(document);
        state
    }
    #[test]
    fn shorten_uri_undeclared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "SELECT * {
               ?a <http://schema.org/name> ?b .
             }"
        ));
        server.state = state;
        let diagnostic = Diagnostic {
            range: Range::new(1, 5, 1, 29),
            severity: diagnostic::DiagnosticSeverity::Information,
            message: String::new(),
            source: None,
            code: None,
            data: Some(LSPAny::LSPArray(vec![
                LSPAny::String("schema".to_string()),
                LSPAny::String("http://schema.org/".to_string()),
                LSPAny::String("schema:name".to_string()),
            ])),
        };

        let code_action = shorten_uri(&mut server, &"uri".to_string(), diagnostic)
            .unwrap()
            .unwrap();

        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(1, 5, 1, 29), "schema:name"),
                TextEdit::new(
                    Range::new(0, 0, 0, 0),
                    "PREFIX schema: <http://schema.org/>\n"
                ),
            ]
        );
    }

    #[test]
    fn shorten_uri_declared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "PREFIX schema: <http://schema.org/>
             SELECT * {
               ?a <http://schema.org/name> ?b .
             }"
        ));
        server.state = state;
        let diagnostic = Diagnostic {
            range: Range::new(2, 5, 2, 29),
            severity: diagnostic::DiagnosticSeverity::Information,
            message: String::new(),
            source: None,
            code: None,
            data: Some(LSPAny::LSPArray(vec![
                LSPAny::String("schema".to_string()),
                LSPAny::String("http://schema.org".to_string()),
                LSPAny::String("schema:name".to_string()),
            ])),
        };

        let code_action = shorten_uri(&mut server, &"uri".to_string(), diagnostic)
            .unwrap()
            .unwrap();
        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
            &vec![TextEdit::new(Range::new(2, 5, 2, 29), "schema:name"),]
        );
    }
}
