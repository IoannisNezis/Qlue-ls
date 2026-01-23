use super::{error::CompletionError, utils::matches_search_term, CompletionEnvironment};
use crate::server::lsp::{
    Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::syntax_kind::SyntaxKind::*;

pub(super) fn completions(
    context: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut items = Vec::new();
    let search_term = context.search_term.as_deref();
    if context.continuations.contains(&SolutionModifier)
        && matches_search_term("GROUP BY", search_term)
    {
        items.push(CompletionItem {
            label: "GROUP BY".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Group the results".to_string()),
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some("GROUP BY $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
        });
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&HavingClause))
        && matches_search_term("HAVING", search_term)
    {
        items.push(CompletionItem {
            command: None,
            label: "HAVING".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Filter Groups".to_string()),
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some("HAVING $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
        });
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OrderClause))
        && matches_search_term("ORDER BY", search_term)
    {
        items.push(CompletionItem {
            label: "ORDER BY".to_string(),
            label_details: None,
            kind: CompletionItemKind::Keyword,
            detail: Some("Sort the results".to_string()),
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some("ORDER BY ".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
        });
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&LimitClause)
        || context.continuations.contains(&LimitOffsetClauses))
        && matches_search_term("LIMIT", search_term)
    {
        items.push(CompletionItem {
            label: "LIMIT".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Limit the results".to_string()),
            documentation: None,
            filter_text: None,
            sort_text: None,
            insert_text: Some("LIMIT ${0:50}".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: None,
        });
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OffsetClause)
        || context.continuations.contains(&LimitOffsetClauses))
        && matches_search_term("OFFSET", search_term)
    {
        items.push(CompletionItem {
            label: "OFFSET".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("OFFSET the results".to_string()),
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some("OFFSET ${0:50}".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: None,
        });
    }
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: Some(ItemDefaults {
            insert_text_format: Some(InsertTextFormat::Snippet),
            data: None,
            commit_characters: None,
            edit_range: None,
            insert_text_mode: None,
        }),
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::matches_search_term;

    const SOLUTION_MODIFIER_KEYWORDS: [&str; 5] =
        ["GROUP BY", "HAVING", "ORDER BY", "LIMIT", "OFFSET"];

    fn filter_keywords(search_term: Option<&str>) -> Vec<&'static str> {
        SOLUTION_MODIFIER_KEYWORDS
            .into_iter()
            .filter(|label| matches_search_term(label, search_term))
            .collect()
    }

    #[test]
    fn no_search_term_returns_all_keywords() {
        let labels = filter_keywords(None);
        assert_eq!(labels.len(), 5);
        assert!(labels.contains(&"GROUP BY"));
        assert!(labels.contains(&"HAVING"));
        assert!(labels.contains(&"ORDER BY"));
        assert!(labels.contains(&"LIMIT"));
        assert!(labels.contains(&"OFFSET"));
    }

    #[test]
    fn group_prefix_returns_group_by() {
        let labels = filter_keywords(Some("GR"));
        assert_eq!(labels, vec!["GROUP BY"]);
    }

    #[test]
    fn having_prefix_returns_having() {
        let labels = filter_keywords(Some("HA"));
        assert_eq!(labels, vec!["HAVING"]);
    }

    #[test]
    fn order_prefix_returns_order_by() {
        let labels = filter_keywords(Some("OR"));
        assert_eq!(labels, vec!["ORDER BY"]);
    }

    #[test]
    fn limit_prefix_returns_limit() {
        let labels = filter_keywords(Some("LI"));
        assert_eq!(labels, vec!["LIMIT"]);
    }

    #[test]
    fn offset_prefix_returns_offset() {
        let labels = filter_keywords(Some("OF"));
        assert_eq!(labels, vec!["OFFSET"]);
    }

    #[test]
    fn o_prefix_returns_order_by_and_offset() {
        let labels = filter_keywords(Some("O"));
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&"ORDER BY"));
        assert!(labels.contains(&"OFFSET"));
    }

    #[test]
    fn non_keyword_prefix_returns_empty() {
        let labels = filter_keywords(Some("Germany"));
        assert!(labels.is_empty());
    }

    #[test]
    fn case_insensitive_matching() {
        let labels = filter_keywords(Some("group"));
        assert_eq!(labels, vec!["GROUP BY"]);
    }
}
