use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::{
    Server,
    lsp::{
        CompletionItemBuilder, CompletionItemKind, CompletionList,
        InsertTextFormat, ItemDefaults,
    },
    message_handler::completion::{CompletionEnvironment, CompletionError, handler::variable},
};

pub async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let variable_completions = variable::completions(server_rc, environment).await?;
    Ok(
        if environment
            .anchor_token
            .as_ref()
            .is_some_and(|anchor| anchor.kind() == SyntaxKind::BY)
        {
            CompletionList {
                is_incomplete: false,
                item_defaults: Some(ItemDefaults {
                    insert_text_format: Some(InsertTextFormat::Snippet),
                    data: None,
                    commit_characters: None,
                    edit_range: None,
                    insert_text_mode: None,
                }),
                items: variable_completions
                    .items
                    .into_iter()
                    .flat_map(|variable_completion| {
                        ["DESC", "ASC"]
                            .into_iter()
                            .map(move |order| (order, variable_completion.label.clone()))
                    })
                    .enumerate()
                    .map(|(idx, (order, var))| {
                        CompletionItemBuilder::new()
                            .label(&format!("{order}({var})"))
                            .kind(CompletionItemKind::Method)
                            .detail(&format!("Order by descending {}", var))
                            .sort_text(&format!("{idx:0>5}"))
                            .insert_text(&format!("{order}({var})"))
                            .build()
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
