use crate::{
    Server,
    server::{
        lsp::{
            CompletionItem, CompletionItemBuilder, CompletionItemKind, CompletionList,
            textdocument::TextEdit,
        },
        message_handler::completion::{
            CompletionEnvironment, CompletionError, CompletionLocation, utils::matches_search_term,
        },
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, GroupClause};
use std::{collections::HashSet, rc::Rc};

pub async fn completions(
    _server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::GroupCondition(group_clause) = &environment.location {
        let variables = collect_variables(group_clause);
        let completions = CompletionList {
            is_incomplete: false,
            item_defaults: None,
            items: variables
                .into_iter()
                .filter(|var| matches_search_term(var, environment.search_term.as_deref()))
                .map(|variable| -> CompletionItem {
                    CompletionItemBuilder::new()
                        .label(&variable)
                        .kind(CompletionItemKind::Variable)
                        .text_edit(TextEdit::new(environment.replace_range.clone(), &variable))
                        .build()
                })
                .collect(),
        };
        Ok(completions)
    } else {
        panic!("group condition completion called for non group condition location");
    }
}

fn collect_variables(group_clause: &GroupClause) -> Vec<String> {
    let grouped_variables_set: HashSet<String> = HashSet::from_iter(
        group_clause
            .visible_variables()
            .into_iter()
            .map(|var| var.text()),
    );
    let projected_variables: Vec<String> = group_clause
        .select_query()
        .map(|sq| {
            sq.visible_variables()
                .into_iter()
                .map(|var| var.text())
                .collect()
        })
        .unwrap_or_default();
    let projected_variables_set: HashSet<String> =
        HashSet::from_iter(projected_variables.iter().cloned());
    let body_variables: Vec<String> = group_clause
        .select_query()
        .and_then(|sq| sq.where_clause())
        .map(|wc| {
            wc.visible_variables()
                .into_iter()
                .map(|var| var.text())
                .collect()
        })
        .unwrap_or_default();
    projected_variables
        .into_iter()
        .chain(
            body_variables
                .into_iter()
                .filter(|var| !projected_variables_set.contains(var)),
        )
        .filter(|var| !grouped_variables_set.contains(var))
        .collect()
}
