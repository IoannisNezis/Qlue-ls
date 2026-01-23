use std::{collections::HashSet, rc::Rc};

use super::{CompletionEnvironment, CompletionLocation, error::CompletionError};
use crate::server::{
    Server,
    configuration::Replacement,
    lsp::{
        Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, PrefixedName, Var, VarOrTerm};
use regex::Regex;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
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
            .map(|var| var.var_name()),
    )
    .into_iter()
    .map(|var| CompletionItem {
        insert_text: Some(format!("{}{}", var, suffix)),
        label: var.clone(),
        label_details: None,
        detail: Some("Variable".to_string()),
        documentation: None,
        kind: CompletionItemKind::Variable,
        sort_text: None,
        filter_text: Some(format!("?{var}")),
        text_edit: None,
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
    // NOTE: predicate based object variable completions:
    if matches!(
        environment.location,
        CompletionLocation::Object(_) | CompletionLocation::BlankNodeObject(_)
    ) {
        if let Some(prefixed_name) = environment
            .anchor_token
            .clone()
            .and_then(|token| token.parent())
            .and_then(PrefixedName::cast)
        {
            let mut object_name = server_rc
                .lock()
                .await
                .state
                .label_memory
                .get(&prefixed_name.text())
                .cloned()
                .unwrap_or(prefixed_name.name());

            if let Some(replacements) = server_rc
                .lock()
                .await
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
            let object_var_name = to_sparql_variable(&object_name);
            suggestions.insert(
                0,
                CompletionItem::new(
                    &object_var_name,
                    None,
                    Some(format!("{:0>5}", 0)),
                    &format!("{}{}", object_var_name, suffix),
                    CompletionItemKind::Variable,
                    None,
                ),
            );
            // NOTE: If subject is a variable:
            // append ?[variable]_[object_name] as variable completion
            if let CompletionLocation::Object(triple) = environment.location.clone() {
                if let Some(var) = triple
                    .subject()
                    .map(|subject| subject.syntax().clone())
                    .and_then(VarOrTerm::cast)
                    .and_then(|var_or_term| var_or_term.var())
                {
                    let subject_var_name = var.var_name();
                    suggestions.insert(
                        0,
                        CompletionItem::new(
                            &format!("{}_{}", subject_var_name, object_var_name),
                            None,
                            Some(format!("{:0>5}", 1)),
                            &format!("{}_{}{}", subject_var_name, object_var_name, suffix),
                            CompletionItemKind::Variable,
                            None,
                        ),
                    );
                }
            }
        }
    }

    // Apply variable completion limit if configured
    let limit = server_rc
        .lock()
        .await
        .settings
        .completion
        .variable_completion_limit;
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
/// - Returns the variable name with '?' prefix
fn to_sparql_variable(s: &str) -> String {
    if s.is_empty() {
        return "?var".to_string();
    }

    // Remove leading '?' or '$' if present
    let s = s
        .strip_prefix('?')
        .or_else(|| s.strip_prefix('$'))
        .unwrap_or(s);

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

pub(super) async fn completions_transformed(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut variable_completions = completions(server_rc, environment).await?;
    for item in variable_completions.items.iter_mut() {
        item.insert_text = item.insert_text.as_ref().map(|text| format!("?{}", text));
        item.label = format!("?{}", item.label);
        item.filter_text = Some(format!("?{}", item.label));
        if item.sort_text.is_none() {
            item.sort_text = Some(format!("{:0>4}0", 1));
        }
    }
    Ok(variable_completions)
}
