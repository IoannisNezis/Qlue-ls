use super::super::{CompletionEnvironment, error::CompletionError, utils::matches_search_term};
use crate::server::lsp::{
    Command, CompletionItemBuilder, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::syntax_kind::SyntaxKind::*;

pub fn completions(context: &CompletionEnvironment) -> Result<CompletionList, CompletionError> {
    let mut items = Vec::new();
    let search_term = context.search_term.as_deref();
    if context.continuations.contains(&SolutionModifier)
        && matches_search_term("GROUP BY", search_term)
    {
        items.push(
            CompletionItemBuilder::new()
                .label("GROUP BY")
                .detail("Group the results")
                .documentation("Group results by one or more expressions. Used with aggregate functions like COUNT, SUM, AVG, MIN, MAX, and GROUP_CONCAT. Each unique combination of the grouped expressions produces one result row.")
                .insert_text("GROUP BY $0")
                .command(Command {
                    title: "triggerNewCompletion".to_string(),
                    command: "triggerNewCompletion".to_string(),
                    arguments: None,
                })
                .build(),
        );
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&HavingClause))
        && matches_search_term("HAVING", search_term)
    {
        items.push(
            CompletionItemBuilder::new()
                .label("HAVING")
                .detail("Filter Groups")
                .documentation("Filter grouped results by applying a condition to aggregated values. HAVING is to GROUP BY what FILTER is to WHERE — it eliminates groups that don't satisfy the condition.")
                .insert_text("HAVING $0")
                .build(),
        );
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OrderClause))
        && matches_search_term("ORDER BY", search_term)
    {
        items.push(
            CompletionItemBuilder::new()
                .label("ORDER BY")
                .detail("sort the results")
                .insert_text("ORDER BY $0")
                .command(Command {
                    title: "triggerNewCompletion".to_string(),
                    command: "triggerNewCompletion".to_string(),
                    arguments: None,
                })
                .build(),
        );
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&LimitClause)
        || context.continuations.contains(&LimitOffsetClauses))
        && matches_search_term("LIMIT", search_term)
    {
        items.push(
            CompletionItemBuilder::new()
                .label("LIMIT")
                .detail("Limit the amount of results")
                .insert_text("LIMIT ${0:50}")
                .build(),
        );
    }
    if (context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OffsetClause)
        || context.continuations.contains(&LimitOffsetClauses))
        && matches_search_term("OFFSET", search_term)
    {
        items.push(
            CompletionItemBuilder::new()
                .label("OFFSET")
                .detail("Drop the first n results")
                .insert_text("OFFSET ${0:50}")
                .build(),
        );
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
