use super::{error::CompletionError, CompletionEnvironment, CompletionLocation};
use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::collections::HashSet;

pub(super) fn completions(
    context: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::SelectBinding(select_clause) = &context.location {
        let mut items = Vec::new();
        // NOTE: suggest keywords DISTINCT & REDUCED
        if context.continuations.contains(&SyntaxKind::DISTINCT) {
            items.append(&mut vec![
                CompletionItem::new(
                    "DISTINCT",
                    Some("Ensure unique results".to_string()),
                    None,
                    "DISTINCT ",
                    CompletionItemKind::Keyword,
                    None,
                ),
                CompletionItem::new(
                    "REDUCED",
                    Some("Permit elimination of some non-distinct solutions".to_string()),
                    None,
                    "REDUCED ",
                    CompletionItemKind::Keyword,
                    None,
                ),
            ]);
        }
        // NOTE: suggest availible variables (non duplicates).
        let result_vars: HashSet<String> = HashSet::from_iter(
            select_clause
                .variables()
                .iter()
                .map(|var| var.syntax().text().to_string()),
        );
        let availible_vars: HashSet<String> =
            select_clause
                .select_query()
                .map_or(HashSet::new(), |select_query| {
                    HashSet::from_iter(
                        select_query
                            .variables()
                            .iter()
                            .map(|var| var.syntax().text().to_string()),
                    )
                });
        let group_vars: HashSet<String> = HashSet::from_iter(
            select_clause
                .select_query()
                .and_then(|sq| sq.soulution_modifier())
                .and_then(|sm| sm.group_clause())
                .map(move |gc| gc.visible_variables().into_iter().map(|var| var.text()))
                .into_iter()
                .flatten(),
        );
        let vars = if group_vars.len() == 0 {
            &availible_vars
        } else {
            &group_vars
        } - &result_vars;
        items.extend(vars.into_iter().map(|var| {
            CompletionItem::new(
                &var,
                Some("variable".to_string()),
                None,
                &format!("{} ", var),
                CompletionItemKind::Variable,
                None,
            )
        }));
        // NOTE: suggest aggregates
        let group_by = select_clause
            .select_query()
            .and_then(|sq| sq.soulution_modifier())
            .and_then(|sm| sm.group_clause());
        // NOTE: If no variables are selected, implicit GROUP BY is allowed.
        if group_by.is_some() || result_vars.len() == 0 {
            let grouped_vars: HashSet<String> =
                HashSet::from_iter(group_by.into_iter().flat_map(|group_by| {
                    group_by
                        .visible_variables()
                        .into_iter()
                        .map(|var| var.syntax().text().to_string())
                }));
            let vars = &availible_vars - &grouped_vars;

            items.extend(
                ["COUNT", "SUM", "MIN", "MAX", "AVG", "SAMPLE"]
                    .into_iter()
                    .map(|aggregate| {
                        vars.iter().map(move |var| CompletionItem {
                            label: format!(
                                "({aggregate}({var}) AS ?{}_{})",
                                aggregate.to_lowercase(),
                                var.split_at(1).1
                            ),
                            label_details: None,
                            kind: CompletionItemKind::Snippet,
                            detail: None,
                            sort_text: None,
                            filter_text: None,
                            insert_text: Some(format!(
                                "({aggregate}(?s) AS ?${{0:{}_{}}})",
                                aggregate.to_lowercase(),
                                var.split_at(1).1
                            )),
                            text_edit: None,
                            insert_text_format: Some(InsertTextFormat::Snippet),
                            additional_text_edits: None,
                            command: None,
                        })
                    })
                    .flatten(),
            );

            items.push(CompletionItem {
                label: "(COUNT(*) AS ?count)".to_string(),
                label_details: None,
                kind: CompletionItemKind::Snippet,
                detail: None,
                sort_text: None,
                filter_text: None,
                insert_text: Some("(COUNT(*) AS ?${0:count})".to_string()),
                text_edit: None,
                insert_text_format: Some(InsertTextFormat::Snippet),
                additional_text_edits: None,
                command: None,
            });
        }

        Ok(CompletionList {
            is_incomplete: false,
            item_defaults: Some(ItemDefaults {
                edit_range: None,
                commit_characters: None,
                data: None,
                insert_text_format: None,
            }),
            items,
        })
    } else {
        Err(CompletionError::Resolve(format!(
            "select binding completions was called with location: {:?}",
            context.location
        )))
    }
}
