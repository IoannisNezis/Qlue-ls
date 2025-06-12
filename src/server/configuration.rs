use std::collections::HashMap;

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

use super::lsp::Backend;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct BackendsSettings {
    pub backends: HashMap<String, BackendConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfiguration {
    pub backend: Backend,
    pub prefix_map: HashMap<String, String>,
    pub default: bool,
    pub queries: Queries,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Queries {
    pub subject_completion: String,
    pub predicate_completion: String,
    pub object_completion: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct CompletionSettings {
    pub timeout_ms: u32,
    pub result_size_limit: u32,
}

impl Default for CompletionSettings {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            result_size_limit: 100,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct FormatSettings {
    pub align_predicates: bool,
    pub align_prefixes: bool,
    pub separate_prolouge: bool,
    pub capitalize_keywords: bool,
    pub insert_spaces: Option<bool>,
    pub tab_size: Option<u8>,
    pub where_new_line: bool,
    pub filter_same_line: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            align_predicates: true,
            align_prefixes: false,
            separate_prolouge: false,
            capitalize_keywords: true,
            insert_spaces: Some(true),
            tab_size: Some(2),
            where_new_line: false,
            filter_same_line: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrefixesSettings {
    pub add_missing: Option<bool>,
    pub remove_unused: Option<bool>,
}

impl Default for PrefixesSettings {
    fn default() -> Self {
        Self {
            add_missing: Some(true),
            remove_unused: Some(true),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Settings {
    /// Format settings
    pub format: FormatSettings,
    /// Completion Settings
    pub completion: CompletionSettings,
    /// Backend configurations
    pub backends: Option<BackendsSettings>,
    /// Automatically add and remove prefix declarations
    pub prefixes: Option<PrefixesSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: FormatSettings::default(),
            completion: CompletionSettings::default(),
            backends: None,
            prefixes: Some(PrefixesSettings::default()),
        }
    }
}

fn load_user_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("qlue-ls"))
        .build()?
        .try_deserialize::<Settings>()
}

impl Settings {
    pub fn new() -> Self {
        match load_user_configuration() {
            Ok(settings) => {
                log::info!("Loaded user configuration!!");
                settings
            }
            Err(error) => {
                log::info!(
                    "Did not load user-configuration:\n{}\n falling back to default values",
                    error
                );
                Settings::default()
            }
        }
    }
}
