use futures::lock::Mutex;
use ll_sparql_parser::{
    SyntaxNode,
    ast::{AstNode, Path, Prologue, QueryUnit},
    syntax_kind::SyntaxKind,
};
use std::rc::Rc;
use tera::Context;
use text_size::TextSize;

use crate::{
    server::{
        Server,
        lsp::{
            BackendService, Command, CompletionItem, CompletionItemKind,
            CompletionItemLabelDetails, CompletionList,
            textdocument::{Position, Range, TextEdit},
        },
        sparql_operations::execute_query,
    },
    sparql::results::RDFTerm,
};

use super::{environment::CompletionEnvironment, error::CompletionError};

/// Returns true if the label matches the search term as a case-insensitive prefix.
/// If no search term is provided (or it's empty), returns true to show all completions.
pub(super) fn matches_search_term(label: &str, search_term: Option<&str>) -> bool {
    match search_term {
        Some(term) if !term.is_empty() => label.to_uppercase().starts_with(&term.to_uppercase()),
        _ => true,
    }
}

pub(super) type CompletionTemplate = crate::server::configuration::CompletionTemplate;

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
                environment.search_term.as_deref(),
            ))
        }
        _ => {
            log::info!("No Backend for completion query found");
            Err(CompletionError::Resolve("No Backend defined".to_string()))
        }
    }
}

pub(super) struct InternalCompletionItem {
    label: String,
    detail: Option<String>,
    value: String,
    _filter_text: Option<String>,
    score: Option<usize>,
    import_edit: Option<TextEdit>,
}

pub(super) async fn fetch_online_completions(
    server_rc: Rc<Mutex<Server>>,
    query_unit: &QueryUnit,
    backend: &BackendService,
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

    let result = execute_query(
        server_rc.clone(),
        url,
        query,
        None,
        None,
        Some(timeout_ms),
        method,
        None,
        0,
        false,
    )
    .await
    .map_err(|err| match err {
        crate::server::sparql_operations::SparqlRequestError::Timeout => {
            CompletionError::Request("Completion query timed out".to_string())
        }
        crate::server::sparql_operations::SparqlRequestError::Connection(_err) => {
            CompletionError::Request("Completion query failed, connection errored".to_string())
        }
        crate::server::sparql_operations::SparqlRequestError::_Canceled(_err) => {
            CompletionError::Request("Completion query was canceled".to_string())
        }
        crate::server::sparql_operations::SparqlRequestError::Response(msg) => {
            CompletionError::Request(msg)
        }
        crate::server::sparql_operations::SparqlRequestError::Deserialization(msg) => {
            CompletionError::Request(msg)
        }
        #[cfg(target_arch = "wasm32")]
        crate::server::sparql_operations::SparqlRequestError::QLeverException(exception) => {
            CompletionError::Request(exception.exception)
        }
    })?
    .expect("Non-lazy request should always return a result.");
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
                .map_or(String::new(), |rdf_term| rdf_term.value().to_string());
            let detail = binding
                .get("qlue_ls_alias")
                .map(|rdf_term: &RDFTerm| rdf_term.value().to_string());
            let score = binding
                .get("qlue_ls_count")
                .and_then(|rdf_term: &RDFTerm| rdf_term.value().parse().ok());
            // NOTE: This is the text the in editor filter uses.
            // If a compressed IRI is used as search term i.e. "wdt:p" the expanded iri is used as
            // filter text. Otherwise the label and detail is used as filter text.
            // This is currently not used, but should be redone at some point
            let filter_text = query_template_context
                .get("search_term_uncompressed")
                .is_some()
                .then_some(value.to_string())
                .or((!label.is_empty()).then_some(format!(
                    "{}{}",
                    label,
                    detail.as_ref().unwrap_or(&String::new())
                )))
                .or(Some(rdf_term.to_string()));
            if !label.is_empty() {
                server
                    .state
                    .label_memory
                    .insert(value.clone(), label.clone());
            }
            InternalCompletionItem {
                label,
                detail,
                value,
                _filter_text: filter_text,
                score,
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
        RDFTerm::Uri { value, curie: _ } => match server.shorten_uri(value, Some(backend_name)) {
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
        start: Position::new(
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
        end: context.trigger_textdocument_position,
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
        SyntaxKind::PathNegatedPropertySet => match path.syntax().last_child() {
            Some(last_child) => reduce_path(subject, &Path::cast(last_child)?, object, offset),
            _ => Some(format!("{} ?qlue_ls_entity {}", subject, object)),
        },
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
    _limit: u32,
    search_term: Option<&str>,
) -> CompletionList {
    let items: Vec<_> = items
        .into_iter()
        .enumerate()
        .map(
            |(
                idx,
                InternalCompletionItem {
                    label,
                    detail,
                    value,
                    _filter_text,
                    score,
                    import_edit,
                },
            )| {
                CompletionItem {
                    label: format!("{value}"),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: format!(
                            "{}{}",
                            &label,
                            detail
                                .as_ref()
                                .map_or(String::new(), |detail| format!("/{detail}"))
                        ),
                    }),
                    detail: None,
                    documentation: Some(format!(
                        "Label: {label}\nAlias: {}\nScore: {}",
                        detail.unwrap_or_default(),
                        score.map_or("None".to_string(), |score| score.to_string()),
                    )),
                    // NOTE: The first 100 ID's are reserved
                    sort_text: Some(format!("{:0>5}", idx + 100)),
                    insert_text: None,
                    // NOTE: Use the search term as filter_text for all items.
                    // This gives all items the same fuzzy match score in Monaco,
                    // forcing it to fall back to sortText for ordering.
                    filter_text: search_term.map(|s| s.to_string()),
                    text_edit: Some(TextEdit {
                        range: range.clone(),
                        new_text: format!("{} ", value),
                    }),
                    kind: CompletionItemKind::Value,
                    insert_text_format: None,
                    additional_text_edits: import_edit.map(|edit| vec![edit]),
                    command: command.map(|command| Command {
                        title: command.to_string(),
                        command: command.to_string(),
                        arguments: None,
                    }),
                }
            },
        )
        .collect();
    CompletionList {
        is_incomplete: true,
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

    use super::{matches_search_term, reduce_path};

    #[test]
    fn matches_search_term_exact_match() {
        assert!(matches_search_term("FILTER", Some("FILTER")));
    }

    #[test]
    fn matches_search_term_prefix_match() {
        assert!(matches_search_term("FILTER", Some("FI")));
        assert!(matches_search_term("FILTER", Some("F")));
        assert!(matches_search_term("OPTIONAL", Some("OP")));
        assert!(matches_search_term("GROUP BY", Some("GR")));
    }

    #[test]
    fn matches_search_term_case_insensitive() {
        assert!(matches_search_term("FILTER", Some("fi")));
        assert!(matches_search_term("FILTER", Some("filter")));
        assert!(matches_search_term("FILTER", Some("Filter")));
        assert!(matches_search_term("OPTIONAL", Some("opt")));
        assert!(matches_search_term("GROUP BY", Some("group")));
    }

    #[test]
    fn matches_search_term_no_match() {
        assert!(!matches_search_term("FILTER", Some("Germany")));
        assert!(!matches_search_term("FILTER", Some("BI")));
        assert!(!matches_search_term("OPTIONAL", Some("FI")));
        assert!(!matches_search_term("BIND", Some("FILTER")));
    }

    #[test]
    fn matches_search_term_none_shows_all() {
        assert!(matches_search_term("FILTER", None));
        assert!(matches_search_term("BIND", None));
        assert!(matches_search_term("OPTIONAL", None));
    }

    #[test]
    fn matches_search_term_empty_string_shows_all() {
        assert!(matches_search_term("FILTER", Some("")));
        assert!(matches_search_term("BIND", Some("")));
        assert!(matches_search_term("OPTIONAL", Some("")));
    }

    #[test]
    fn matches_search_term_partial_word_not_prefix() {
        // "ILTER" is not a prefix of "FILTER"
        assert!(!matches_search_term("FILTER", Some("ILTER")));
        // "TER" is not a prefix of "FILTER"
        assert!(!matches_search_term("FILTER", Some("TER")));
    }

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
