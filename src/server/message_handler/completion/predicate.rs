use std::collections::HashSet;

use super::{
    utils::{fetch_online_completions, get_prefix_declarations, get_replace_range},
    CompletionContext,
};
use crate::server::{
    lsp::CompletionItem, message_handler::completion::context::CompletionLocation, Server,
};
use ll_sparql_parser::{
    ast::{AstNode, Path, QueryUnit, Triple},
    syntax_kind::SyntaxKind,
};
use tera::Context;
use text_size::TextSize;

static QUERY_TEMPLATE: &str = "predicate_completion.rq";

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    if let CompletionLocation::Predicate(triple) = &context.location {
        let range = get_replace_range(&context);
        let mut template_context = Context::new();
        let query_unit = QueryUnit::cast(context.tree.clone()).unwrap();
        let prefixes = get_prefix_declarations(server, &context, triple);
        if let Some(inject) = compute_inject_context(
            triple,
            context.anchor_token.unwrap().text_range().end(),
            context.continuations,
        ) {
            template_context.insert("context", &inject);
        } else {
            return vec![];
        }
        template_context.insert("prefixes", &prefixes);
        match fetch_online_completions(
            server,
            &query_unit,
            context.backend.as_ref(),
            QUERY_TEMPLATE,
            template_context,
            range,
        )
        .await
        {
            Ok(online_completions) => online_completions,
            Err(err) => {
                log::error!("{:?}", err);
                vec![]
            }
        }
    } else {
        panic!("object completions requested for non object location");
    }
}

fn compute_inject_context(
    triple: &Triple,
    offset: TextSize,
    continuations: HashSet<SyntaxKind>,
) -> Option<String> {
    let subject_string = triple.subject()?.text();
    if continuations.contains(&SyntaxKind::PropertyListPath)
        || continuations.contains(&SyntaxKind::PropertyListPathNotEmpty)
    {
        Some(format!("{} ?qlue_ls_value ?qlue_ls_inner2", subject_string))
    } else {
        let properties = triple.properties_list_path()?.properties();
        if continuations.contains(&SyntaxKind::VerbPath) {
            Some(format!("{} ?qlue_ls_value ?qlue_ls_inner2", triple.text()))
        } else if properties.len() == 1 {
            reduce_path(
                &subject_string,
                &properties[0].verb,
                "?qlue_ls_inner2",
                offset,
            )
        } else {
            let (last_prop, prev_prop) = properties.split_last()?;
            Some(format!(
                "{} {} . {}",
                subject_string,
                prev_prop
                    .iter()
                    .map(|prop| prop.text())
                    .collect::<Vec<_>>()
                    .join(" ; "),
                reduce_path(&subject_string, &last_prop.verb, "?qlue_ls_inner2", offset)?
            ))
        }
    }
}

fn reduce_path(subject: &str, path: &Path, object: &str, offset: TextSize) -> Option<String> {
    if path.syntax().text_range().start() >= offset {
        return Some(format!("{} ?qlue_ls_value {}", subject, object));
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
                Some(format!("{} ?qlue_ls_value {}", subject, object))
            }
        }
        SyntaxKind::PathOneInPropertySet => {
            let first_child = path.syntax().first_child_or_token()?;
            if first_child.kind() == SyntaxKind::Zirkumflex {
                if first_child.text_range().end() == offset {
                    Some(format!("{} ?qlue_ls_value {}", object, subject))
                } else {
                    Some(format!("{} ?qlue_ls_value {}", subject, object))
                }
            } else {
                Some(path.text().to_string())
            }
        }
        _ => panic!("unknown path kind"),
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
        let reduced = "?a <p0> ?qlue_ls_inner . ?qlue_ls_inner ?qlue_ls_value ?qlue_ls_inner2";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_alternating_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a <p0>/<p1>|  <x>}";
        let reduced = "?a ?qlue_ls_value ?qlue_ls_inner2";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_inverse_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a ^  <x>}";
        let reduced = "?qlue_ls_inner2 ?qlue_ls_value ?a";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_negated_path() {
        //       012345678901234567890123456
        let s = "Select * { ?a !()}";
        let reduced = "?a ?qlue_ls_value ?qlue_ls_inner2";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path1() {
        //       0123456789012345678901234567890123456
        let s = "Select * { ?a <p0>|<p1>/(<p2>)/^  <x>}";
        let reduced =
            "?a <p1>/(<p2>) ?qlue_ls_inner . ?qlue_ls_inner2 ?qlue_ls_value ?qlue_ls_inner";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }
    #[test]
    fn reduce_complex_path2() {
        //       01234567890123456789012345678901234567890
        let s = "Select * { ?a <p0>|<p1>/(<p2>)/^<p2>/!(^)  <x>}";
        let reduced =
            "?a <p1>/(<p2>)/^<p2> ?qlue_ls_inner . ?qlue_ls_inner2 ?qlue_ls_value ?qlue_ls_inner";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path3() {
        //       0123456789012345678901234567890123456
        let s = "Select * { ?a ^(^<a>/)  <x>}";
        let reduced = "?qlue_ls_inner2 ^<a> ?qlue_ls_inner . ?qlue_ls_inner ?qlue_ls_value ?a";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }

    #[test]
    fn reduce_complex_path4() {
        //       01234567890123456
        let s = "Select * { ?a !^  <x>}";
        let reduced = "?qlue_ls_inner2 ?qlue_ls_value ?a";
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
            "?qlue_ls_inner2",
            offset.into(),
        )
        .unwrap();
        assert_eq!(res, reduced);
    }
}
