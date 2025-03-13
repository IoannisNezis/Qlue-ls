use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    if context.continuations.contains(&SyntaxKind::DISTINCT) {
        vec![
            CompletionItem::new(
                "DISTINCT",
                "Ensure unique results",
                "DISTINCT ",
                CompletionItemKind::Keyword,
                InsertTextFormat::PlainText,
            ),
            CompletionItem::new(
                "REDUCED",
                "Permit elimination of some non-distinct solutions",
                "REDUCED ",
                CompletionItemKind::Keyword,
                InsertTextFormat::PlainText,
            ),
        ]
    } else {
        vec![]
    }
}
