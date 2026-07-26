use std::{collections::HashSet, rc::Rc};

use super::super::{CompletionEnvironment, CompletionLocation, error::CompletionError};
use crate::server::{
    Server,
    lsp::{
        Command, CompletionItem, CompletionItemBuilder, CompletionItemKind, CompletionList,
        InsertTextFormat, ItemDefaults, textdocument::TextEdit,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, PrefixedName, Var, VarOrTerm};

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
            .full_tree
            .descendants()
            .filter_map(Var::cast)
            .filter(|var| !var.has_error())
            .map(|var| format!("?{}", var.var_name()))
            .filter(|var| environment.search_term.as_ref().is_some_and(|st| st != var)),
    )
    .into_iter()
    .map(|var| {
        let mut item = CompletionItemBuilder::new()
            .label(&var)
            .kind(CompletionItemKind::Variable)
            .detail("Variable")
            .sort_text(&format!("{:0>4}0", 1))
            .filter_text(&var)
            .text_edit(TextEdit::new(
                environment.replace_range.clone(),
                &format!("{var}{suffix}"),
            ))
            .insert_text_format(InsertTextFormat::PlainText)
            .build();
        match environment.location {
            CompletionLocation::Subject
            | CompletionLocation::Predicate(_)
            | CompletionLocation::BlankNodeProperty(_) => {
                item.command = Some(Command {
                    title: "triggerNewCompletion".to_string(),
                    command: "triggerNewCompletion".to_string(),
                    arguments: None,
                });
            }
            _ => {}
        }
        item
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
        if let Some(replacements) = server.settings.replacements.as_ref() {
            object_name = replacements.apply_object_variable(&object_name);
        }
        let variable = to_sparql_variable(&object_name);
        suggestions.insert(
            0,
            CompletionItemBuilder::new()
                .label(&format!("?{variable}"))
                .kind(CompletionItemKind::Variable)
                .sort_text("00000")
                .filter_text(&format!("?{variable}"))
                .text_edit(TextEdit::new(
                    environment.replace_range.clone(),
                    &format!("?{variable}"),
                ))
                .insert_text_format(InsertTextFormat::PlainText)
                .build(),
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
                CompletionItemBuilder::new()
                    .label(&format!("?{}_{}", subject_var_name, variable))
                    .kind(CompletionItemKind::Variable)
                    .sort_text("00001")
                    .filter_text(&format!("?{}_{}", subject_var_name, variable))
                    .text_edit(TextEdit::new(
                        environment.replace_range.clone(),
                        &format!("?{}_{}", subject_var_name, variable),
                    ))
                    .insert_text_format(InsertTextFormat::PlainText)
                    .build(),
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
                CompletionItemBuilder::new()
                    .label(&format!("?{}_{}", subject_label, variable))
                    .kind(CompletionItemKind::Variable)
                    .sort_text("00001")
                    .filter_text(&format!("?{}_{}", subject_label, variable))
                    .text_edit(TextEdit::new(
                        environment.replace_range.clone(),
                        &format!("?{}_{}", subject_label, variable),
                    ))
                    .insert_text_format(InsertTextFormat::PlainText)
                    .build(),
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

#[cfg(test)]
mod tests {
    use super::to_sparql_variable;
    use crate::server::configuration::Replacements;

    /// The full object variable name pipeline: configured replacements first,
    /// then the built in snake_case conversion.
    fn pipeline(name: &str) -> String {
        to_sparql_variable(&Replacements::default().apply_object_variable(name))
    }

    #[test]
    fn test_to_sparql_variable_converts_to_snake_case() {
        assert_eq!(to_sparql_variable("BirthDate"), "birthdate");
        assert_eq!(to_sparql_variable("birth date"), "birth_date");
        assert_eq!(to_sparql_variable("part-of"), "part_of");
        assert_eq!(to_sparql_variable("place.of.birth"), "place_of_birth");
        assert_eq!(to_sparql_variable("label (english)"), "label_english");
    }

    #[test]
    fn test_to_sparql_variable_strips_variable_prefix() {
        assert_eq!(to_sparql_variable("?author"), "author");
        assert_eq!(to_sparql_variable("$author"), "author");
    }

    #[test]
    fn test_to_sparql_variable_produces_valid_names() {
        assert_eq!(to_sparql_variable(""), "var");
        assert_eq!(to_sparql_variable("   "), "var");
        assert_eq!(to_sparql_variable("2020"), "_2020");
        assert_eq!(to_sparql_variable("_private"), "_private");
        assert_eq!(to_sparql_variable("a/b"), "a_b");
    }

    #[test]
    fn test_to_sparql_variable_trims_and_collapses_whitespace() {
        assert_eq!(to_sparql_variable("  date of  birth "), "date_of_birth");
    }

    #[test]
    fn test_pipeline_with_default_replacements() {
        assert_eq!(pipeline("hasAuthor"), "author");
        assert_eq!(pipeline("authoredBy"), "author");
        assert_eq!(pipeline("has author"), "author");
        assert_eq!(pipeline("P31"), "p31");
    }

    #[test]
    fn test_pipeline_keeps_word_boundaries_as_snake_case() {
        // NOTE: multi word names keep their boundaries, whether they are
        // separated by whitespace, punctuation or camelCase.
        assert_eq!(pipeline("has birth date"), "birth_date");
        assert_eq!(pipeline("place of birth"), "place_of_birth");
        assert_eq!(pipeline("hasBirthDate"), "birth_date");
        assert_eq!(pipeline("birthDate"), "birth_date");
        assert_eq!(pipeline("place-of-birth"), "place_of_birth");
        assert_eq!(pipeline("hasPlaceOfBirth"), "place_of_birth");
    }

    #[test]
    fn test_pipeline_produces_valid_names_for_degenerate_input() {
        assert_eq!(pipeline("///"), "var");
        assert_eq!(pipeline(""), "var");
        assert_eq!(pipeline("P31/P279*"), "p31_p279");
    }
}
