use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, Path, QueryUnit},
    syntax_kind::SyntaxKind,
};
use std::rc::Rc;
use tera::Context;
use text_size::TextSize;
use wasm_bindgen::JsCast;

use crate::{
    server::{
        fetch::fetch_sparql_result,
        lsp::{
            textdocument::{Position, Range, TextEdit},
            Command, CompletionItem, CompletionItemKind, CompletionItemLabelDetails,
        },
        Server,
    },
    sparql::results::RDFTerm,
};

use super::{context::CompletionContext, error::CompletionError};

pub(super) async fn fetch_online_completions(
    server_rc: Rc<Mutex<Server>>,
    query_unit: &QueryUnit,
    backend_name: Option<&String>,
    query_template: &str,
    mut query_template_context: Context,
) -> Result<Vec<(Option<String>, Option<String>, String, Option<TextEdit>)>, CompletionError> {
    let (url, query, timeout_ms) = {
        let server = server_rc.lock().await;
        query_template_context.insert("limit", &server.settings.completion.result_size_limit);
        query_template_context.insert("offset", &0);
        let query = server
            .tools
            .tera
            .render(query_template, &query_template_context)
            .map_err(|err| CompletionError::TemplateError(query_template.to_string(), err))?;

        let backend = backend_name.ok_or(CompletionError::ResolveError(
            "Could not resolve online completion, no backend provided.".to_string(),
        ))?;
        let url = server
            .state
            .get_backend(backend)
            .ok_or(CompletionError::ResolveError(
                "No default SPARQL backend defined".to_string(),
            ))?
            .url
            .clone();
        let timeout_ms = server.settings.completion.timeout_ms;
        (url, query, timeout_ms)
    };
    let performance = js_sys::global()
        .unchecked_into::<web_sys::WorkerGlobalScope>()
        .performance()
        .unwrap();
    log::debug!("Query:\n{}", query);
    let start = performance.now();
    let result = fetch_sparql_result(&url, &query, timeout_ms)
        .await
        .map_err(|err| CompletionError::RequestError(err.message))?;

    let end = performance.now();
    log::debug!(
        "Query took {:?}ms, returned {} results",
        (end - start) as i32,
        result.results.bindings.len()
    );
    let server = server_rc.lock().await;
    Ok(result
        .results
        .bindings
        .into_iter()
        .map(|binding| {
            let rdf_term = binding
                .get("qlue_ls_entity")
                .expect("Every completion query should provide a `qlue_ls_entity`");
            let (value, import_edit) =
                render_rdf_term(&*server, query_unit, rdf_term, backend_name);
            let label = binding
                .get("qlue_ls_label")
                .map(|rdf_term| rdf_term.value().to_string());
            let detail = binding
                .get("qlue_ls_detail")
                .map(|rdf_term: &RDFTerm| rdf_term.value().to_string());
            (label, detail, value, import_edit)
        })
        .collect())
}

fn render_rdf_term(
    server: &Server,
    query_unit: &QueryUnit,
    rdf_term: &RDFTerm,
    backend_name: Option<&String>,
) -> (String, Option<TextEdit>) {
    match rdf_term {
        RDFTerm::Uri { value } => match server.shorten_uri(value, backend_name) {
            Some((prefix, uri, curie)) => {
                let prefix_decl_edit = if query_unit.prologue().as_ref().map_or(true, |prologue| {
                    prologue
                        .prefix_declarations()
                        .iter()
                        .all(|prefix_declaration| {
                            prefix_declaration
                                .prefix()
                                .map_or(false, |declared_prefix| declared_prefix != prefix)
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
pub(super) fn get_replace_range(context: &CompletionContext) -> Range {
    Range {
        start: context.trigger_textdocument_position,
        end: Position::new(
            context.trigger_textdocument_position.line,
            context.trigger_textdocument_position.character
                - context
                    .search_term
                    .as_ref()
                    .expect("search_term should be Some")
                    .chars()
                    .fold(0, |accu, char| accu + char.len_utf16()) as u32,
        ),
    }
}

pub(super) fn get_prefix_declarations<'a>(
    server: &Server,
    context: &CompletionContext,
    prefixes: Vec<String>,
) -> Vec<(String, String)> {
    prefixes
        .into_iter()
        .filter_map(|prefix| {
            context.backend.as_ref().and_then(|backend| {
                server
                    .state
                    .get_converter(backend)
                    .and_then(|converter| converter.find_by_prefix(&prefix).ok())
            })
        })
        .map(|record| (record.prefix.clone(), record.uri_prefix.clone()))
        .collect()
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
    items: Vec<(Option<String>, Option<String>, String, Option<TextEdit>)>,
    range: Range,
    command: Option<&str>,
) -> Vec<CompletionItem> {
    items
        .into_iter()
        .enumerate()
        .map(
            |(idx, (label, detail, value, import_edit))| CompletionItem {
                label: format!("{} ", label.as_ref().unwrap_or(&value)),
                label_details: detail
                    .map(|detail| CompletionItemLabelDetails { detail })
                    .or(label.and(Some(CompletionItemLabelDetails {
                        detail: value.clone(),
                    }))),
                detail: None,
                sort_text: Some(format!("{:0>5}", idx + 100)),
                insert_text: None,
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
            },
        )
        .collect()
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
