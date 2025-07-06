use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::{
    lsp::{CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults},
    message_handler::completion::{variable, CompletionEnvironment, CompletionError},
    Server,
};

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let variable_completions = variable::completions_transformed(server_rc, &environment).await?;
    Ok(
        if environment
            .anchor_token
            .is_some_and(|anchor| anchor.kind() == SyntaxKind::BY)
        {
            CompletionList {
                is_incomplete: false,
                item_defaults: Some(ItemDefaults {
                    insert_text_format: Some(InsertTextFormat::Snippet),
                    data: None,
                    commit_characters: None,
                    edit_range: None,
                }),
                items: variable_completions
                    .items
                    .into_iter()
                    .map(|variable_completion| {
                        ["DESC", "ASC"]
                            .into_iter()
                            .map(move |order| (order, variable_completion.label.clone()))
                    })
                    .flatten()
                    .enumerate()
                    .map(|(idx, (order, var))| CompletionItem {
                        label: format!("{order}({var})"),
                        label_details: None,
                        kind: CompletionItemKind::Method,
                        detail: Some(format!("Order by descending {}", var)),
                        sort_text: Some(format!("{idx:0>5}")),
                        filter_text: None,
                        insert_text: Some(format!("{order}({var})")),
                        text_edit: None,
                        insert_text_format: None,
                        additional_text_edits: None,
                        command: None,
                    })
                    .collect(),
            }
        } else {
            CompletionList {
                is_incomplete: false,
                item_defaults: None,
                items: Vec::new(),
            }
        },
    )
}
