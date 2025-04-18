use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CompletionSettings {
    pub timeout_ms: u32,
    pub result_size_limit: u32,
}

impl Default for CompletionSettings {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            result_size_limit: 42,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
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
            insert_spaces: None,
            tab_size: None,
            where_new_line: false,
            filter_same_line: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub format: FormatSettings,
    pub completion: CompletionSettings,
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
                log::info!("Loaded user configuration\n{:?}", settings);
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
