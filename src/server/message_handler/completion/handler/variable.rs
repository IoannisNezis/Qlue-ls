use std::{collections::HashSet, rc::Rc};

use super::super::{CompletionEnvironment, CompletionLocation, error::CompletionError};
use crate::server::{
    Server,
    configuration::Replacement,
    lsp::{
        Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat,
        ItemDefaults, textdocument::TextEdit,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, PrefixedName, Var, VarOrTerm};
use regex::Regex;

pub async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let server = server_rc.lock().await;

    let suffix = match environment.location {
        CompletionLocation::Object(_)
        | CompletionLocation::Subject
        | CompletionLocation::Predicate(_)
        | CompletionLocation::BlankNodeProperty(_)
        | CompletionLocation::BlankNodeObject(_) => " ",
        _ => "",
    };
    let mut suggestions: Vec<CompletionItem> = HashSet::<String>::from_iter(
        environment
            .tree
            .descendants()
            .filter_map(Var::cast)
            .map(|var| format!("?{}", var.var_name())),
    )
    .into_iter()
    .map(|var| CompletionItem {
        label: var.clone(),
        label_details: None,
        kind: CompletionItemKind::Variable,
        detail: Some("Variable".to_string()),
        documentation: None,
        sort_text: Some(format!("{:0>4}0", 1)),
        filter_text: Some(var.clone()),
        insert_text: None,
        text_edit: Some(TextEdit::new(
            environment.replace_range.clone(),
            &format!("{var}{suffix}"),
        )),
        insert_text_format: Some(InsertTextFormat::PlainText),
        additional_text_edits: None,
        command: match environment.location {
            CompletionLocation::Subject
            | CompletionLocation::Predicate(_)
            | CompletionLocation::BlankNodeProperty(_) => Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
            _ => None,
        },
    })
    .collect();
    // NOTE: augmented object variable completions:
    if matches!(
        environment.location,
        CompletionLocation::Object(_) | CompletionLocation::BlankNodeObject(_)
    ) && let Some(prefixed_name) = environment
        .anchor_token
        .clone()
        .and_then(|token| token.parent())
        .and_then(PrefixedName::cast)
    {
        // NOTE: what should the variable be called based on the predicate.
        let mut object_name = server
            .state
            .label_memory
            .get(&prefixed_name.text())
            .cloned()
            .unwrap_or(prefixed_name.name());

        // NOTE: Apply "replacements.
        // For example:
        // "has_author" -> "author"
        // These replacements are configurable.
        // snace_case conversion is build in.
        if let Some(replacements) = server
            .settings
            .replacements
            .as_ref()
            .map(|replacements| &replacements.object_variable)
        {
            for Replacement {
                pattern,
                replacement,
            } in replacements.iter()
            {
                object_name = Regex::new(pattern)
                    .unwrap()
                    .replace_all(&object_name, replacement)
                    .to_string();
            }
        }
        let variable = to_sparql_variable(&object_name);
        suggestions.insert(
            0,
            CompletionItem {
                label: format!("?{variable}"),
                label_details: None,
                kind: CompletionItemKind::Variable,
                detail: None,
                documentation: None,
                sort_text: Some("00000".to_string()),
                filter_text: Some(format!("?{variable}")),
                insert_text: None,
                text_edit: Some(TextEdit::new(
                    environment.replace_range.clone(),
                    &format!("?{variable}"),
                )),
                insert_text_format: Some(InsertTextFormat::PlainText),
                additional_text_edits: None,
                command: None,
            },
        );
        // NOTE: If subject is a variable:
        // append ?[variable]_[object_name] as variable completion
        if let CompletionLocation::Object(ref triple) = environment.location
            && let Some(var) = triple
                .subject()
                .map(|subject| subject.syntax().clone())
                .and_then(VarOrTerm::cast)
                .and_then(|var_or_term| var_or_term.var())
        {
            let subject_var_name = var.var_name();
            suggestions.insert(
                0,
                CompletionItem {
                    label: format!("?{}_{}", subject_var_name, variable),
                    label_details: None,
                    kind: CompletionItemKind::Variable,
                    detail: None,
                    documentation: None,
                    sort_text: Some("00001".to_string()),
                    filter_text: Some(format!("?{}_{}", subject_var_name, variable)),
                    insert_text: None,
                    text_edit: Some(TextEdit::new(
                        environment.replace_range.clone(),
                        &format!("?{}_{}", subject_var_name, variable),
                    )),
                    insert_text_format: Some(InsertTextFormat::PlainText),
                    additional_text_edits: None,
                    command: None,
                },
            );
        }

        // NOTE: If subject has a known label:
        // append ?[label]_[object_name] as variable completion,
        // where label is rewritten in snake_case.
        if let CompletionLocation::Object(ref triple) = environment.location
            && let Some(subject_label) = triple
                .subject()
                .map(|subject| subject.syntax().to_string())
                .and_then(|subject| server.state.label_memory.get(&subject))
        {
            let subject_label = to_sparql_variable(subject_label);
            suggestions.insert(
                0,
                CompletionItem {
                    label: format!("?{}_{}", subject_label, variable),
                    label_details: None,
                    kind: CompletionItemKind::Variable,
                    detail: None,
                    documentation: None,
                    sort_text: Some("00001".to_string()),
                    filter_text: Some(format!("?{}_{}", subject_label, variable)),
                    insert_text: None,
                    text_edit: Some(TextEdit::new(
                        environment.replace_range.clone(),
                        &format!("?{}_{}", subject_label, variable),
                    )),
                    insert_text_format: Some(InsertTextFormat::PlainText),
                    additional_text_edits: None,
                    command: None,
                },
            );
        }
    }

    // Apply variable completion limit if configured
    let limit = server.settings.completion.variable_completion_limit;
    if let Some(limit) = limit {
        suggestions.truncate(limit as usize);
    }

    Ok(CompletionList {
        is_incomplete: limit.is_some_and(|l| suggestions.len() >= l as usize),
        item_defaults: Some(ItemDefaults {
            edit_range: None,
            commit_characters: None,
            data: None,
            insert_text_format: Some(InsertTextFormat::PlainText),
            insert_text_mode: None,
        }),
        items: suggestions,
    })
}

/// Transforms an arbitrary string into a valid SPARQL variable name.
///
/// SPARQL variable names must:
/// - Start with a letter (A-Z, a-z) or underscore
/// - Contain only letters, digits, underscores
/// - Be prefixed with '?' or '$'
///
/// This function:
/// - Removes the '?' or '$' prefix if present
/// - Replaces invalid characters with underscores
/// - Ensures the name starts with a valid character
/// - Returns the variable name WITHOUT '?' or '$' prefix
fn to_sparql_variable(s: &str) -> String {
    if s.is_empty() {
        return "var".to_string();
    }

    // NOTE: first pass: convert into snace_case
    let s = &s
        .trim()
        .to_lowercase()
        .replace([' ', '-', ',', '.', '(', ')', '$', '?'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");

    let mut result = String::new();
    let mut chars = s.chars();

    // Handle first character - must be letter or underscore
    if let Some(first) = chars.next() {
        if first.is_ascii_alphabetic() || first == '_' {
            result.push(first);
        } else if first.is_ascii_digit() {
            // If starts with digit, prefix with underscore
            result.push('_');
            result.push(first);
        } else {
            // Replace invalid first char with underscore
            result.push('_');
        }
    }

    // Process remaining characters
    for c in chars {
        if c.is_ascii_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
        }
    }

    // Ensure we have at least some content
    if result.is_empty() {
        result.push_str("var");
    }
    result
}
