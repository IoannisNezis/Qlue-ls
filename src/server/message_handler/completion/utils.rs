use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, Path, Prologue, QueryUnit},
    syntax_kind::SyntaxKind,
    SyntaxNode,
};
use std::{fmt::Display, rc::Rc};
use tera::Context;
use text_size::TextSize;

use crate::{
    server::{
        fetch::fetch_sparql_result,
        lsp::{
            textdocument::{Position, Range, TextEdit},
            Backend, Command, CompletionItem, CompletionItemKind, CompletionItemLabelDetails,
            CompletionList,
        },
        Server,
    },
    sparql::results::RDFTerm,
};

use super::{environment::CompletionEnvironment, error::CompletionError};

pub(super) enum CompletionTemplate {
    SubjectCompletion,
    PredicateCompletion,
    ObjectCompletion,
    PredicateCompletionContextSensitive,
    ObjectCompletionContextSensitive,
}

impl Display for CompletionTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionTemplate::SubjectCompletion => write!(f, "subjectCompletion"),
            CompletionTemplate::PredicateCompletion => write!(f, "predicateCompletion"),
            CompletionTemplate::ObjectCompletion => write!(f, "objectCompletion"),
            CompletionTemplate::PredicateCompletionContextSensitive => {
                write!(f, "predicateCompletionContextSensitive")
            }
            CompletionTemplate::ObjectCompletionContextSensitive => {
                write!(f, "objectCompletionContextSensitive")
            }
        }
    }
}

pub(super) async fn dispatch_completion_query(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
    template_context: Context,
    completion_template: CompletionTemplate,
    trigger_on_accept: bool,
) -> Result<CompletionList, CompletionError> {
    match environment.backend.as_ref() {
        Some(backend) => {
            let query_unit = QueryUnit::cast(environment.tree.clone()).ok_or(
                CompletionError::Resolve("Could not cast root to QueryUnit".to_string()),
            )?;
            Ok(to_completion_items(
                fetch_online_completions(
                    server_rc.clone(),
                    &query_unit,
                    backend,
                    &format!("{}-{}", backend.name, completion_template),
                    template_context,
                )
                .await?,
                get_replace_range(environment),
                trigger_on_accept.then_some("triggerNewCompletion"),
                server_rc.lock().await.settings.completion.result_size_limit,
            ))
        }
        _ => {
            log::info!("No Backend for completion query found");
            Err(CompletionError::Resolve("No Backend defined".to_string()))
        }
    }
}

pub(super) struct InternalCompletionItem {
    label: Option<String>,
    detail: Option<String>,
    value: String,
    filter_text: Option<String>,
    import_edit: Option<TextEdit>,
}

pub(super) async fn fetch_online_completions(
    server_rc: Rc<Mutex<Server>>,
    query_unit: &QueryUnit,
    backend: &Backend,
    query_template: &str,
    mut query_template_context: Context,
) -> Result<Vec<InternalCompletionItem>, CompletionError> {
    let (url, query, timeout_ms, method) = {
        let server = server_rc.lock().await;
        query_template_context.insert("limit", &server.settings.completion.result_size_limit);
        query_template_context.insert("offset", &0);
        let query = server
            .tools
            .tera
            .render(query_template, &query_template_context)
            .map_err(|err| CompletionError::Template(query_template.to_string(), err))?;

        let url = backend.url.clone();
        let timeout_ms = server.settings.completion.timeout_ms;
        let method = server.state.get_backend_request_method(&backend.name);
        (url, query, timeout_ms, method)
    };

    log::debug!("Query:\n{}", query);
    let result = fetch_sparql_result(&url, &query, timeout_ms, method)
        .await
        .map_err(|err| match err {
            crate::server::fetch::SparqlRequestError::Timeout => {
                CompletionError::Request("Completion query timed out".to_string())
            }
            crate::server::fetch::SparqlRequestError::Connection => {
                CompletionError::Request("Completion query failed, connection errored".to_string())
            }
            crate::server::fetch::SparqlRequestError::Response(msg) => {
                CompletionError::Request(msg)
            }
            crate::server::fetch::SparqlRequestError::Deserialization(msg) => {
                CompletionError::Request(msg)
            }
        })?;
    log::info!("Result size: {}", result.results.bindings.len());

    let mut server = server_rc.lock().await;
    Ok(result
        .results
        .bindings
        .into_iter()
        .map(|binding| {
            let rdf_term = binding
                .get("qlue_ls_entity")
                .expect("Every completion query should provide a `qlue_ls_entity`");
            let (value, import_edit) =
                render_rdf_term(&server, query_unit, rdf_term, &backend.name);
            let label = binding
                .get("qlue_ls_label")
                .map(|rdf_term| rdf_term.value().to_string());
            let detail = binding
                .get("qlue_ls_detail")
                .map(|rdf_term: &RDFTerm| rdf_term.value().to_string());
            let filter_text = query_template_context
                .get("search_term_uncompressed")
                .is_some()
                .then_some(value.to_string());
            if let Some(label) = label.as_ref() {
                server
                    .state
                    .label_memory
                    .insert(value.clone(), label.clone());
            }
            InternalCompletionItem {
                label,
                detail,
                value,
                filter_text,
                import_edit,
            }
        })
        .collect())
}

fn render_rdf_term(
    server: &Server,
    query_unit: &QueryUnit,
    rdf_term: &RDFTerm,
    backend_name: &str,
) -> (String, Option<TextEdit>) {
    match rdf_term {
        RDFTerm::Uri { value } => match server.shorten_uri(value, Some(backend_name)) {
            Some((prefix, uri, curie)) => {
                let prefix_decl_edit = if query_unit.prologue().as_ref().is_none_or(|prologue| {
                    prologue
                        .prefix_declarations()
                        .iter()
                        .all(|prefix_declaration| {
                            prefix_declaration
                                .prefix()
                                .is_some_and(|declared_prefix| declared_prefix != prefix)
                        })
                }) {
                    Some(TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri),
                    ))
                } else {
                    None
                };
                (curie, prefix_decl_edit)
            }
            None => (rdf_term.to_string(), None),
        },
        _ => (rdf_term.to_string(), None),
    }
}

/// Get the range the completion is supposed to replace
/// The context.search_term MUST be not None!
pub(super) fn get_replace_range(context: &CompletionEnvironment) -> Range {
    Range {
        start: context.trigger_textdocument_position,
        end: Position::new(
            context.trigger_textdocument_position.line,
            context.trigger_textdocument_position.character
                - context
                    .search_term
                    .as_ref()
                    .map(|search_term| {
                        search_term
                            .chars()
                            .fold(0, |accu, char| accu + char.len_utf16())
                            as u32
                    })
                    .unwrap_or(0),
        ),
    }
}

pub(super) async fn get_prefix_declarations(root: &SyntaxNode) -> Vec<(String, String)> {
    root.first_child()
        .and_then(|child| child.first_child())
        .and_then(Prologue::cast)
        .map(|prologue| {
            prologue
                .prefix_declarations()
                .iter()
                .filter_map(|prefix_declaration| {
                    match (
                        prefix_declaration.prefix(),
                        prefix_declaration.raw_uri_prefix(),
                    ) {
                        (Some(prefix), Some(uri_prefix)) => Some((prefix, uri_prefix)),
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn reduce_path(
    subject: &str,
    path: &Path,
    object: &str,
    offset: TextSize,
) -> Option<String> {
    if path.syntax().text_range().start() >= offset {
        return Some(format!("{} ?qlue_ls_entity {}", subject, object));
    }
    match path.syntax().kind() {
        SyntaxKind::PathPrimary | SyntaxKind::PathElt | SyntaxKind::Path | SyntaxKind::VerbPath => {
            reduce_path(
                subject,
                &Path::cast(path.syntax().first_child()?)?,
                object,
                offset,
            )
        }
        SyntaxKind::PathAlternative => {
            reduce_path(subject, &path.sub_paths().last()?, object, offset)
        }
        SyntaxKind::PathSequence => {
            let sub_paths = path
                .sub_paths()
                .map(|sub_path| sub_path.text())
                .collect::<Vec<_>>();
            let path_seq_len = sub_paths.len();
            if path_seq_len > 1 {
                let path_prefix = sub_paths[..path_seq_len - 1].join("/");
                let prefix = format!("{} {} {}", subject, path_prefix, "?qlue_ls_inner");
                Some(format!(
                    "{} . {}",
                    prefix,
                    reduce_path("?qlue_ls_inner", &path.sub_paths().last()?, object, offset)?
                ))
            } else {
                reduce_path(subject, &path.sub_paths().last()?, object, offset)
            }
        }
        SyntaxKind::PathEltOrInverse => {
            if path.syntax().first_child_or_token()?.kind() == SyntaxKind::Zirkumflex {
                reduce_path(
                    object,
                    &Path::cast(path.syntax().last_child()?)?,
                    subject,
                    offset,
                )
            } else {
                reduce_path(
                    subject,
                    &Path::cast(path.syntax().last_child()?)?,
                    object,
                    offset,
                )
            }
        }
        SyntaxKind::PathNegatedPropertySet => {
            if let Some(last_child) = path.syntax().last_child() {
                reduce_path(subject, &Path::cast(last_child)?, object, offset)
            } else {
                Some(format!("{} ?qlue_ls_entity {}", subject, object))
            }
        }
        SyntaxKind::PathOneInPropertySet => {
            let first_child = path.syntax().first_child_or_token()?;
            if first_child.kind() == SyntaxKind::Zirkumflex {
                if first_child.text_range().end() == offset {
                    Some(format!("{} ?qlue_ls_entity {}", object, subject))
                } else {
                    Some(format!("{} ?qlue_ls_entity {}", subject, object))
                }
            } else {
                Some(path.text().to_string())
            }
        }
        _ => panic!("unknown path kind"),
    }
}

pub(super) fn to_completion_items(
    items: Vec<InternalCompletionItem>,
    range: Range,
    command: Option<&str>,
    limit: u32,
) -> CompletionList {
    let items: Vec<_> = items
        .into_iter()
        .enumerate()
        .map(|(idx, internal_completion_item)| CompletionItem {
            label: format!(
                "{} ",
                internal_completion_item
                    .label
                    .as_ref()
                    .unwrap_or(&internal_completion_item.value)
            ),
            label_details: internal_completion_item
                .detail
                .map(|detail| CompletionItemLabelDetails { detail })
                .or(internal_completion_item
                    .label
                    .and(Some(CompletionItemLabelDetails {
                        detail: internal_completion_item.value.clone(),
                    }))),
            detail: None,
            // NOTE: The first 100 id's. are reserved
            sort_text: Some(format!("{:0>5}", idx + 100)),
            insert_text: None,
            filter_text: internal_completion_item.filter_text,
            text_edit: Some(TextEdit {
                range: range.clone(),
                new_text: format!("{} ", internal_completion_item.value),
            }),
            kind: CompletionItemKind::Value,
            insert_text_format: None,
            additional_text_edits: internal_completion_item.import_edit.map(|edit| vec![edit]),
            command: command.map(|command| Command {
                title: command.to_string(),
                command: command.to_string(),
                arguments: None,
            }),
        })
        .collect();
    CompletionList {
        is_incomplete: items.len() == limit as usize,
        items,
        item_defaults: None,
    }
}

#[cfg(test)]
mod test {
    use ll_sparql_parser::{
        ast::{AstNode, QueryUnit},
        parse_query,
    };

    use super::reduce_path;

    #[test]
    fn reduce_sequence_path() {
        //       0123456789012345678901
        let s = "Select * { ?a <p0>/  }";
        let reduced = "?a <p0> ?qlue_ls_inner . ?qlue_ls_inner ?qlue_ls_entity []";
        let offset = 19;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_alternating_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a <p0>/<p1>|  <x>}";
        let reduced = "?a ?qlue_ls_entity []";
        let offset = 24;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_inverse_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a ^  <x>}";
        let reduced = "[] ?qlue_ls_entity ?a";
        let offset = 15;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_negated_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a !()}";
        let reduced = "?a ?qlue_ls_entity []";
        let offset = 16;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path1() {
        //       0123456789012345678901234567890123456
        let s = "Select * { ?a <p0>|<p1>/(<p2>)/^  <x>}";
        let reduced = "?a <p1>/(<p2>) ?qlue_ls_inner . [] ?qlue_ls_entity ?qlue_ls_inner";
        let offset = 32;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }
    #[test]
    fn reduce_complex_path2() {
        //       01234567890123456789012345678901234567890
        let s = "Select * { ?a <p0>|<p1>/(<p2>)/^<p2>/!(^)  <x>}";
        let reduced = "?a <p1>/(<p2>)/^<p2> ?qlue_ls_inner . [] ?qlue_ls_entity ?qlue_ls_inner";
        let offset = 40;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path3() {
        //       0123456789012345678901234567890123456
        let s = "Select * { ?a ^(^<a>/)  <x>}";
        let reduced = "[] ^<a> ?qlue_ls_inner . ?qlue_ls_inner ?qlue_ls_entity ?a";
        let offset = 21;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path4() {
        //       01234567890123456
        let s = "Select * { ?a !^  <x>}";
        let reduced = "[] ?qlue_ls_entity ?a";
        let offset = 16;
        let query_unit = QueryUnit::cast(parse_query(s)).unwrap();
        let triples = query_unit
            .select_query()
            .unwrap()
            .where_clause()
            .unwrap()
            .group_graph_pattern()
            .unwrap()
            .triple_blocks()
            .first()
            .unwrap()
            .triples();
        let triple = triples.first().unwrap();
        let res = reduce_path(
            &triple.subject().unwrap().text(),
            &triple
                .properties_list_path()
                .unwrap()
                .properties()
                .last()
                .unwrap()
                .verb,
            "[]",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }
}
