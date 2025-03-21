use ll_sparql_parser::syntax_kind::SyntaxKind::*;

use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    let mut res = Vec::new();
    if context.continuations.contains(&SolutionModifier) {
        res.push(CompletionItem::new(
            "GROUP BY",
            "Group the results",
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
            "Filter Groups",
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
            "Sort the results",
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
            "Limit the results",
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
            "OFFSET the results",
            "OFFSET $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ));
    }
    return res;
}
