use std::rc::Rc;

use futures::lock::Mutex;

use serde_json::Value;

use crate::server::{
    Server,
    configuration::Settings,
    lsp::{
        ChangeSettingsNotification, DefaultSettingsRequest, DefaultSettingsResponse,
        errors::{ErrorCode, LSPError},
    },
};

#[tracing::instrument(skip_all, fields(id = %request.base.id))]
pub(super) async fn handle_default_settings_request(
    server_rc: Rc<Mutex<Server>>,
    request: DefaultSettingsRequest,
) -> Result<(), LSPError> {
    server_rc
        .lock()
        .await
        .send_message(DefaultSettingsResponse::new(
            request.base.id,
            Settings::default(),
        ))
}

/// Merges the received settings into the current ones.
///
/// The notification payload is a partial settings object: every key the client
/// sends overrides the current value, every key it omits is kept.
#[tracing::instrument(skip_all)]
pub(super) async fn handle_change_settings_notification(
    server_rc: Rc<Mutex<Server>>,
    request: ChangeSettingsNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    server.settings = merge_settings(&server.settings, request.params)?;
    tracing::info!("Updated settings: {:?}", server.settings);
    Ok(())
}

/// Applies the partial settings object `patch` on top of `settings`.
fn merge_settings(settings: &Settings, patch: Value) -> Result<Settings, LSPError> {
    let mut merged = serde_json::to_value(settings).map_err(|error| {
        LSPError::new(
            ErrorCode::InternalError,
            &format!("Could not serialize the current settings: {}", error),
        )
    })?;
    merge_value(&mut merged, patch);
    serde_json::from_value(merged).map_err(|error| {
        LSPError::new(
            ErrorCode::InvalidParams,
            &format!("Could not apply the received settings: {}", error),
        )
    })
}

/// Recursively merges `patch` into `target`.
///
/// Objects are merged key by key, every other value replaces the target.
/// INFO: arrays are replaced, not concatenated. Appending to a list like
/// `replacements.objectVariable` would make the configured defaults impossible
/// to remove.
fn merge_value(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                match target.get_mut(&key) {
                    Some(target_value) => merge_value(target_value, value),
                    None => {
                        target.insert(key, value);
                    }
                }
            }
        }
        (target, patch) => *target = patch,
    }
}

#[cfg(test)]
mod tests {
    use super::merge_settings;
    use crate::server::configuration::{Replacement, Replacements, Settings};
    use serde_json::json;

    #[test]
    fn test_empty_patch_keeps_everything() {
        let merged = merge_settings(&Settings::default(), json!({})).unwrap();

        assert_eq!(merged, Settings::default());
    }

    #[test]
    fn test_settings_survive_a_serialization_round_trip() {
        // NOTE: merging goes through JSON, so every field must round trip.
        let merged = merge_settings(&Settings::default(), json!(null));

        assert!(merged.is_err(), "a non object patch should be rejected");

        let merged = merge_settings(&Settings::default(), json!({})).unwrap();
        assert_eq!(merged.format, Settings::default().format);
        assert_eq!(merged.completion, Settings::default().completion);
        assert_eq!(merged.prefixes, Settings::default().prefixes);
        assert_eq!(merged.replacements, Settings::default().replacements);
    }

    #[test]
    fn test_patch_overrides_only_the_keys_it_contains() {
        let merged = merge_settings(
            &Settings::default(),
            json!({ "format": { "alignPredicates": false } }),
        )
        .unwrap();

        assert!(!merged.format.align_predicates);
        // Siblings of the changed key are kept.
        assert_eq!(
            merged.format.line_length,
            Settings::default().format.line_length
        );
        assert!(merged.format.capitalize_keywords);
        // Unrelated sections are kept.
        assert_eq!(merged.completion, Settings::default().completion);
        assert_eq!(merged.replacements, Settings::default().replacements);
    }

    #[test]
    fn test_patch_keeps_replacements_when_absent() {
        let merged = merge_settings(
            &Settings::default(),
            json!({ "completion": { "timeoutMs": 1 } }),
        )
        .unwrap();

        assert_eq!(merged.completion.timeout_ms, 1);
        assert_eq!(
            merged.replacements,
            Some(Replacements::default()),
            "an absent `replacements` key must not drop the defaults"
        );
    }

    #[test]
    fn test_patch_replaces_the_replacement_list() {
        // INFO: arrays are replaced, not appended to, otherwise the defaults
        // could never be removed.
        let merged = merge_settings(
            &Settings::default(),
            json!({
                "replacements": {
                    "objectVariable": [{ "pattern": "^is(\\w+)", "replacement": "$1" }]
                }
            }),
        )
        .unwrap();

        assert_eq!(
            merged.replacements.unwrap().object_variable,
            vec![Replacement::new(r"^is(\w+)", "$1")]
        );
    }

    #[test]
    fn test_patch_can_empty_the_replacement_list() {
        let merged = merge_settings(
            &Settings::default(),
            json!({ "replacements": { "objectVariable": [] } }),
        )
        .unwrap();

        assert_eq!(
            merged.replacements,
            Some(Replacements {
                object_variable: vec![]
            })
        );
    }

    #[test]
    fn test_patch_can_disable_replacements_with_null() {
        let merged = merge_settings(&Settings::default(), json!({ "replacements": null })).unwrap();

        assert_eq!(merged.replacements, None);
    }

    #[test]
    fn test_successive_patches_accumulate() {
        let settings = merge_settings(
            &Settings::default(),
            json!({ "replacements": { "objectVariable": [] } }),
        )
        .unwrap();
        let settings =
            merge_settings(&settings, json!({ "format": { "lineLength": 80 } })).unwrap();

        assert_eq!(settings.format.line_length, 80);
        assert_eq!(
            settings.replacements,
            Some(Replacements {
                object_variable: vec![]
            }),
            "the earlier change must survive the later one"
        );
    }

    #[test]
    fn test_patch_merges_nested_backends() {
        let settings = merge_settings(
            &Settings::default(),
            json!({
                "backends": {
                    "backends": {
                        "wikidata": {
                            "name": "Wikidata",
                            "url": "https://query.wikidata.org/sparql",
                            "default": true
                        }
                    }
                }
            }),
        )
        .unwrap();

        // A later patch touching one backend key keeps the rest of it.
        let settings = merge_settings(
            &settings,
            json!({
                "backends": { "backends": { "wikidata": { "default": false } } }
            }),
        )
        .unwrap();

        let backends = settings.backends.expect("backends should be kept");
        let wikidata = backends.backends.get("wikidata").unwrap();
        assert_eq!(wikidata.name, "Wikidata");
        assert_eq!(wikidata.url, "https://query.wikidata.org/sparql");
        assert!(!wikidata.default);
    }

    #[test]
    fn test_invalid_patch_is_rejected() {
        let error = merge_settings(
            &Settings::default(),
            json!({ "format": { "lineLength": "wide" } }),
        )
        .expect_err("a wrongly typed value should be rejected");

        assert!(
            error
                .message
                .contains("Could not apply the received settings")
        );
    }

    #[test]
    fn test_unknown_keys_are_ignored() {
        let merged = merge_settings(&Settings::default(), json!({ "notASetting": 42 })).unwrap();

        assert_eq!(merged, Settings::default());
    }
}
