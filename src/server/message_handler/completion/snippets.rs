use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

pub(super) fn get_union_snippets() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new(
            "SELECT",
            "Select query",
            "SELECT ${1:*} WHERE {\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "PREFIX",
            "Declare a namespace",
            "PREFIX ${1:namespace}: <${0:iri}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "BASE",
            "Set the Base URI",
            "BASE <${0}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
    ]
}

pub(super) fn get_solution_mod_snippets() -> Vec<CompletionItem> {
    vec![
        CompletionItem::new(
            "ORDER BY",
            "Sort the results",
            "ORDER BY ${1|ASC,DESC|} ( $0 )",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "GROUP BY",
            "Group the results",
            "GROUP BY $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "HAVING",
            "Filter Groups",
            "HAVING $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "LIMIT",
            "Limit the results",
            "LIMIT $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
        CompletionItem::new(
            "OFFSET",
            "OFFSET the results",
            "OFFSET $0",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        ),
    ]
}
