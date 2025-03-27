use ll_sparql_parser::syntax_kind::SyntaxKind::*;

use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    let mut res = Vec::new();
    if context.continuations.contains(&SolutionModifier) {
        res.push(CompletionItem::new(
            "GROUP BY",
            Some("Group the results".to_string()),
            None,
            "GROUP BY $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&HavingClause)
    {
        res.push(CompletionItem::new(
            "HAVING",
            Some("Filter Groups".to_string()),
            None,
            "HAVING $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OrderClause)
    {
        res.push(CompletionItem::new(
            "ORDER BY",
            Some("Sort the results".to_string()),
            None,
            "ORDER BY ${1|ASC,DESC|} ( $0 )",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&LimitClause)
        || context.continuations.contains(&LimitOffsetClauses)
    {
        res.push(CompletionItem::new(
            "LIMIT",
            Some("Limit the results".to_string()),
            None,
            "LIMIT $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OffsetClause)
        || context.continuations.contains(&LimitOffsetClauses)
    {
        res.push(CompletionItem::new(
            "OFFSET",
            Some("OFFSET the results".to_string()),
            None,
            "OFFSET $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    return res;
}
