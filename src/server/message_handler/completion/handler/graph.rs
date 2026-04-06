use crate::server::lsp::{
    CompletionItemBuilder, CompletionItemKind, CompletionList, InsertTextFormat,
};

use super::{super::environment::CompletionEnvironment, super::error::CompletionError};

pub fn completions(_context: &CompletionEnvironment) -> Result<CompletionList, CompletionError> {
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: None,
        items: vec![
            CompletionItemBuilder::new()
                .label("<graph>")
                .kind(CompletionItemKind::Value)
                .insert_text("<graph>")
                .insert_text_format(InsertTextFormat::PlainText)
                .build(),
        ],
    })
}
