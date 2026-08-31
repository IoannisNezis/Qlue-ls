//! Server configuration and settings structures.
//!
//! This module defines the configuration schema for qlue-ls, loadable from
//! `qlue-ls.toml` or `qlue-ls.yml` files in the working directory.
//!
//! # Key Types
//!
//! - [`Settings`]: Top-level configuration container
//! - [`FormatSettings`]: Formatter options (alignment, capitalization, spacing)
//! - [`CompletionSettings`]: Timeout and result limits for completions
//! - [`BackendConfiguration`]: SPARQL endpoint with prefix map and custom queries
//!
//! # Configuration Loading
//!
//! [`Settings::new`] attempts to load from a config file. If not found or invalid,
//! it falls back to [`Settings::default`]. Settings can also be updated at runtime
//! via the `qlueLs/changeSettings` notification.
//!
//! # Backend Configuration
//!
//! Backends define SPARQL endpoints used for completions and query execution.
//! Each backend can have:
//! - Custom prefix maps for URI compression
//! - Request method (GET/POST)
//! - Custom SPARQL templates for completion queries
//!
//! # Related Modules
//!
//! - [`super::Server`]: Stores settings in `Server.settings`
//! - [`super::message_handler::settings`]: Handles runtime settings changes

use std::{collections::HashMap, fmt, sync::OnceLock};

#[cfg(not(target_arch = "wasm32"))]
use config::{Config, ConfigError};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::server::lsp::{SparqlEngine, base_types::LSPAny};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct BackendsSettings {
    pub backends: HashMap<String, BackendConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfiguration {
    pub name: String,
    pub url: String,
    pub health_check_url: Option<String>,
    pub engine: Option<SparqlEngine>,
    pub request_method: Option<RequestMethod>,
    #[serde(default)]
    pub prefix_map: HashMap<String, String>,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub queries: HashMap<CompletionTemplate, String>,
    pub additional_data: Option<LSPAny>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "String")]
pub(crate) enum CompletionTemplate {
    Hover,
    SubjectCompletion,
    PredicateCompletionContextSensitive,
    PredicateCompletionContextInsensitive,
    ObjectCompletionContextSensitive,
    ObjectCompletionContextInsensitive,
    ValuesCompletionContextSensitive,
    ValuesCompletionContextInsensitive,
}

#[derive(Debug)]
pub struct UnknownTemplateError(String);

impl fmt::Display for UnknownTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown completion query template \"{}\"", &self.0)
    }
}

impl TryFrom<String> for CompletionTemplate {
    type Error = UnknownTemplateError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "hover" => Ok(CompletionTemplate::Hover),
            "subjectCompletion" => Ok(CompletionTemplate::SubjectCompletion),
            "predicateCompletionContextInsensitive" => {
                Ok(CompletionTemplate::PredicateCompletionContextInsensitive)
            }
            "predicateCompletionContextSensitive" => {
                Ok(CompletionTemplate::PredicateCompletionContextSensitive)
            }
            "objectCompletionContextInsensitive" => {
                Ok(CompletionTemplate::ObjectCompletionContextInsensitive)
            }
            "objectCompletionContextSensitive" => {
                Ok(CompletionTemplate::ObjectCompletionContextSensitive)
            }
            "valuesCompletionContextSensitive" => {
                Ok(CompletionTemplate::ValuesCompletionContextSensitive)
            }
            "valuesCompletionContextInsensitive" => {
                Ok(CompletionTemplate::ValuesCompletionContextInsensitive)
            }
            _ => Err(UnknownTemplateError(s.to_string())),
        }
    }
}

impl fmt::Display for CompletionTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompletionTemplate::Hover => write!(f, "hover"),
            CompletionTemplate::SubjectCompletion => write!(f, "subjectCompletion"),
            CompletionTemplate::PredicateCompletionContextSensitive => {
                write!(f, "predicateCompletionContextSensitive")
            }
            CompletionTemplate::PredicateCompletionContextInsensitive => {
                write!(f, "predicateCompletionContextInsensitive")
            }
            CompletionTemplate::ObjectCompletionContextSensitive => {
                write!(f, "objectCompletionContextSensitive")
            }
            CompletionTemplate::ObjectCompletionContextInsensitive => {
                write!(f, "objectCompletionContextInsensitive")
            }
            CompletionTemplate::ValuesCompletionContextSensitive => {
                write!(f, "valuesCompletionContextSensitive")
            }
            CompletionTemplate::ValuesCompletionContextInsensitive => {
                write!(f, "valuesCompletionContextInsensitive")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum RequestMethod {
    GET,
    POST,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct CompletionSettings {
    pub timeout_ms: u32,
    pub result_size_limit: u32,
    pub subject_completion_trigger_length: u32,
    pub object_completion_suffix: bool,
    /// Maximum number of variable completions to suggest. None means unlimited.
    pub variable_completion_limit: Option<u32>,
    /// When completing a subject that matches the previous triple's subject,
    /// transform the completion to use semicolon notation instead of starting a new triple.
    pub same_subject_semicolon: bool,
}

impl Default for CompletionSettings {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            result_size_limit: 100,
            subject_completion_trigger_length: 3,
            object_completion_suffix: true,
            variable_completion_limit: None,
            same_subject_semicolon: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct FormatSettings {
    pub align_predicates: bool,
    pub align_prefixes: bool,
    pub separate_prologue: bool,
    pub capitalize_keywords: bool,
    pub insert_spaces: Option<bool>,
    pub tab_size: Option<u8>,
    pub where_new_line: bool,
    pub filter_same_line: bool,
    pub compact: Option<u32>,
    pub line_length: u32,
    pub contract_triples: bool,
    /// When enabled, preserves intentional blank lines from the original source.
    /// Consecutive blank lines are collapsed into a single empty line.
    /// Disabled by default to preserve current behavior.
    pub keep_empty_lines: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            align_predicates: true,
            align_prefixes: false,
            separate_prologue: false,
            capitalize_keywords: true,
            insert_spaces: Some(true),
            tab_size: Some(2),
            where_new_line: false,
            filter_same_line: true,
            compact: None,
            line_length: 120,
            contract_triples: false,
            keep_empty_lines: false,
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
            remove_unused: Some(false),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Replacement {
    pub pattern: String,
    pub replacement: String,
    /// Cache for the compiled [`Self::pattern`].
    ///
    /// INFO: not part of the configuration format, it is skipped in both
    /// directions and starts out empty.
    #[serde(skip)]
    regex: OnceLock<Regex>,
}

// NOTE: `Regex` has no `PartialEq`, and the cache is derived from `pattern`
// anyway, so only the configured values take part in the comparison.
impl PartialEq for Replacement {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.replacement == other.replacement
    }
}

impl Replacement {
    pub fn new(pattern: &str, replacement: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            regex: OnceLock::new(),
        }
    }

    /// The compiled [`Self::pattern`], compiled once and cached afterwards.
    ///
    /// Returns an error for an invalid pattern. Patterns come from user
    /// configuration, so this is a normal outcome and not a bug.
    pub fn regex(&self) -> Result<&Regex, regex::Error> {
        if let Some(regex) = self.regex.get() {
            return Ok(regex);
        }
        let regex = Regex::new(&self.pattern)?;
        Ok(self.regex.get_or_init(|| regex))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Replacements {
    pub object_variable: Vec<Replacement>,
}

impl Replacements {
    /// Applies the configured `object_variable` replacements to `name`, in order.
    ///
    /// Each replacement is a regex `pattern` and a `replacement` string that may
    /// reference capture groups (`$1`, ...). Replacements are applied
    /// sequentially, so a later pattern sees the output of the earlier ones.
    ///
    /// WARNING: an invalid pattern is skipped with a warning, the remaining
    /// replacements are still applied. Use [`Self::validate`] to reject invalid
    /// patterns where they enter the server instead.
    pub fn apply_object_variable(&self, name: &str) -> String {
        let mut name = name.to_string();
        for replacement in self.object_variable.iter() {
            match replacement.regex() {
                Ok(regex) => {
                    name = regex
                        .replace_all(&name, &replacement.replacement)
                        .to_string();
                }
                Err(error) => tracing::warn!(
                    "Skipping object variable replacement with invalid pattern \"{}\": {}",
                    replacement.pattern,
                    error
                ),
            }
        }
        name
    }

    /// Checks that every configured pattern compiles.
    ///
    /// Compiled patterns are cached, so a successful validation also warms the
    /// cache for the following [`Self::apply_object_variable`] calls.
    pub fn validate(&self) -> Result<(), String> {
        for replacement in self.object_variable.iter() {
            replacement.regex().map_err(|error| {
                format!(
                    "invalid objectVariable pattern \"{}\": {}",
                    replacement.pattern, error
                )
            })?;
        }
        Ok(())
    }
}

impl Default for Replacements {
    fn default() -> Self {
        Self {
            object_variable: vec![
                // NOTE: strip the "has" prefix and the "edBy" suffix.
                // These run before the camelCase split below, because they
                // match on the camelCase boundary themselves.
                Replacement::new(r"^has (\w+)", "$1"),
                Replacement::new(r"^has([A-Z]\w*)", "$1"),
                Replacement::new(r"^(\w+)edBy", "$1"),
                // NOTE: turn camelCase boundaries into snake_case separators,
                // so "birthDate" becomes "birth_date" and not "birthdate".
                // INFO: `${1}` instead of `$1`, otherwise "$1_" would be read
                // as a reference to a capture group named "1_".
                Replacement::new(r"([a-z0-9])([A-Z])", "${1}_${2}"),
                // NOTE: collapse runs of invalid characters into a single
                // separator instead of dropping them, so multi word names keep
                // their word boundaries.
                Replacement::new(r"[^a-zA-Z0-9_]+", "_"),
                // NOTE: a leading or trailing separator is never meaningful.
                Replacement::new(r"^_+|_+$", ""),
            ],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Format settings
    #[serde(default)]
    pub format: FormatSettings,
    /// Completion Settings
    #[serde(default)]
    pub completion: CompletionSettings,
    /// Backend configurations
    pub backends: Option<BackendsSettings>,
    /// Automatically add and remove prefix declarations
    pub prefixes: Option<PrefixesSettings>,
    /// Automatically add and remove prefix declarations
    pub replacements: Option<Replacements>,
    /// Automatically insert a line break after typing `;` or `.` following a valid triple.
    #[serde(default)]
    pub auto_line_break: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: FormatSettings::default(),
            completion: CompletionSettings::default(),
            backends: None,
            prefixes: Some(PrefixesSettings::default()),
            replacements: Some(Replacements::default()),
            auto_line_break: false,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_user_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("qlue-ls"))
        .build()?
        .try_deserialize::<Settings>()
}

impl Settings {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        match load_user_configuration() {
            Ok(settings) => {
                tracing::info!("Loaded user configuration!!");
                // NOTE: an invalid pattern does not invalidate the whole
                // configuration file, it is skipped when the replacements are
                // applied. Warn about it once here instead of on every
                // completion request.
                if let Some(Err(error)) = settings.replacements.as_ref().map(Replacements::validate)
                {
                    tracing::warn!("Ignoring a replacement from the user configuration: {error}");
                }
                settings
            }
            Err(error) => {
                tracing::info!(
                    "Did not load user-configuration:\n{}\n falling back to default values",
                    error
                );
                Settings::default()
            }
        }
        #[cfg(target_arch = "wasm32")]
        Settings::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, FileFormat};

    fn parse_yaml<T: serde::de::DeserializeOwned>(yaml: &str) -> T {
        Config::builder()
            .add_source(config::File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn test_backend_configuration_valid_queries_all_variants() {
        let yaml = r#"
            name: TestBackend
            url: https://example.com/sparql
            healthCheckUrl: https://example.com/health
            requestMethod: GET
            prefixMap:
              rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
              rdfs: http://www.w3.org/2000/01/rdf-schema#
            default: false
            queries:
              subjectCompletion: SELECT ?qls_entity ?qls_label ?qls_detail WHERE { ?qls_entity a ?type }
              predicateCompletionContextSensitive: SELECT ?qls_entity WHERE { ?s ?qls_entity ?o }
              predicateCompletionContextInsensitive: SELECT ?qls_entity WHERE { [] ?qls_entity [] }
              objectCompletionContextSensitive: SELECT ?qls_entity WHERE { ?s ?p ?qls_entity }
              objectCompletionContextInsensitive: SELECT ?qls_entity WHERE { [] [] ?qls_entity }
              valuesCompletionContextSensitive: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
              valuesCompletionContextInsensitive: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.name, "TestBackend");
        assert_eq!(config.url, "https://example.com/sparql");
        assert!(!config.default);
        assert_eq!(config.queries.len(), 7);
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::SubjectCompletion)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::PredicateCompletionContextSensitive)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::PredicateCompletionContextInsensitive)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::ObjectCompletionContextSensitive)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::ObjectCompletionContextInsensitive)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::ValuesCompletionContextSensitive)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::ValuesCompletionContextInsensitive)
        );
    }

    #[test]
    fn test_backend_configuration_queries_subset() {
        let yaml = r#"
            name: MinimalBackend
            url: https://example.com/sparql
            prefixMap: {}
            queries:
              subjectCompletion: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
              objectCompletionContextInsensitive: SELECT ?qls_entity WHERE { ?s ?p ?qls_entity }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.queries.len(), 2);
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::SubjectCompletion)
        );
        assert!(
            config
                .queries
                .contains_key(&CompletionTemplate::ObjectCompletionContextInsensitive)
        );
        assert!(
            !config
                .queries
                .contains_key(&CompletionTemplate::PredicateCompletionContextSensitive)
        );
    }

    #[test]
    fn test_backend_configuration_rejects_invalid_query_key() {
        // This test ensures that invalid query keys are rejected
        let yaml = r#"
            name: TestBackend
            url: https://example.com/sparql
            prefixMap: {}
            queries:
              invalidQueryType: SELECT ?qls_entity WHERE { ?s ?p ?o }
              subjectCompletion: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
        "#;

        let result = Config::builder()
            .add_source(config::File::from_str(yaml, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<BackendConfiguration>();
        assert!(result.is_err());
    }

    #[test]
    fn test_backend_configuration_with_multiline_queries() {
        let yaml = r#"
            name: WikidataBackend
            url: https://query.wikidata.org/sparql
            healthCheckUrl: https://query.wikidata.org/
            prefixMap:
              wd: http://www.wikidata.org/entity/
              wdt: http://www.wikidata.org/prop/direct/
              rdfs: http://www.w3.org/2000/01/rdf-schema#
            default: false
            queries:
              subjectCompletion: |
                SELECT ?qls_entity ?qls_label ?qls_detail
                WHERE {
                  ?qls_entity rdfs:label ?qls_label .
                  OPTIONAL { ?qls_entity schema:description ?qls_detail }
                  FILTER(LANG(?qls_label) = "en")
                }
                LIMIT 100
              predicateCompletionContextSensitive: |
                SELECT ?qls_entity WHERE {
                  ?s ?qls_entity ?o
                }
              objectCompletionContextInsensitive: SELECT ?qls_entity WHERE { [] [] ?qls_entity }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.name, "WikidataBackend");
        assert_eq!(config.url, "https://query.wikidata.org/sparql");
        assert!(!config.default);
        assert_eq!(config.prefix_map.len(), 3);
        assert_eq!(config.queries.len(), 3);

        // Verify multiline query was parsed correctly
        let subject_query = config
            .queries
            .get(&CompletionTemplate::SubjectCompletion)
            .unwrap();
        assert!(subject_query.contains("SELECT ?qls_entity ?qls_label ?qls_detail"));
        assert!(subject_query.contains("FILTER(LANG(?qls_label) = \"en\")"));
    }

    #[test]
    fn test_backends_settings_multiple_backends() {
        let yaml = r#"
            backends:
              wikidata:
                name: Wikidata
                url: https://query.wikidata.org/sparql
                prefixMap:
                  wd: http://www.wikidata.org/entity/
                queries:
                  subjectCompletion: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
              dbpedia:
                name: DBpedia
                url: https://dbpedia.org/sparql
                prefixMap:
                  dbo: http://dbpedia.org/ontology/
                default: true
                queries:
                  objectCompletionContextSensitive: SELECT ?qls_entity WHERE { ?s ?p ?qls_entity }
        "#;

        let settings: BackendsSettings = parse_yaml(yaml);

        assert_eq!(settings.backends.len(), 2);
        assert!(settings.backends.contains_key("wikidata"));
        assert!(settings.backends.contains_key("dbpedia"));

        let wikidata = settings.backends.get("wikidata").unwrap();
        assert_eq!(wikidata.name, "Wikidata");
        assert_eq!(wikidata.queries.len(), 1);

        let dbpedia = settings.backends.get("dbpedia").unwrap();
        assert_eq!(dbpedia.name, "DBpedia");
        assert!(dbpedia.default);
    }

    #[test]
    fn test_full_settings_deserialization() {
        let yaml = r#"
            format:
              alignPredicates: true
              alignPrefixes: false
              separatePrologue: false
              capitalizeKeywords: true
              insertSpaces: true
              tabSize: 2
              whereNewLine: false
              filterSameLine: true
            completion:
              timeoutMs: 5000
              resultSizeLimit: 100
            backends:
              backends:
                wikidata:
                  name: Wikidata
                  url: https://query.wikidata.org/sparql
                  healthCheckUrl: https://query.wikidata.org/
                  prefixMap:
                    wd: http://www.wikidata.org/entity/
                    wdt: http://www.wikidata.org/prop/direct/
                  default: true
                  queries:
                    subjectCompletion: SELECT ?qls_entity WHERE { ?qls_entity ?p ?o }
                    predicateCompletionContextSensitive: SELECT ?qls_entity WHERE { ?s ?qls_entity ?o }
            prefixes:
              addMissing: true
              removeUnused: false
        "#;

        let settings: Settings = parse_yaml(yaml);

        assert!(settings.format.align_predicates);
        assert_eq!(settings.completion.timeout_ms, 5000);
        assert!(settings.backends.is_some());

        let backends = settings.backends.unwrap();
        assert_eq!(backends.backends.len(), 1);

        let wikidata = backends.backends.get("wikidata").unwrap();
        assert_eq!(wikidata.name, "Wikidata");
        assert!(wikidata.default);
        assert_eq!(wikidata.queries.len(), 2);
    }

    // NOTE: object variable replacements

    #[test]
    fn test_default_replacements_are_valid_regexes() {
        assert_eq!(Replacements::default().validate(), Ok(()));
    }

    #[test]
    fn test_validate_reports_an_invalid_pattern() {
        let replacements = Replacements {
            object_variable: vec![
                Replacement::new(r"^has(\w+)", "$1"),
                Replacement::new(r"([unclosed", ""),
            ],
        };

        let error = replacements
            .validate()
            .expect_err("an unparsable pattern should be reported");

        assert!(
            error.contains("([unclosed"),
            "the error should name the offending pattern, got: {}",
            error
        );
    }

    #[test]
    fn test_invalid_pattern_is_skipped_instead_of_panicking() {
        let replacements = Replacements {
            object_variable: vec![
                Replacement::new(r"([unclosed", ""),
                Replacement::new(r"^has(\w+)", "$1"),
            ],
        };

        // WARNING: this used to panic.
        assert_eq!(replacements.apply_object_variable("hasAuthor"), "Author");
    }

    #[test]
    fn test_only_the_invalid_pattern_is_skipped() {
        let replacements = Replacements {
            object_variable: vec![
                Replacement::new(r"^has(\w+)", "$1"),
                Replacement::new(r"*nope", ""),
                Replacement::new(r"Author", "Creator"),
            ],
        };

        assert_eq!(replacements.apply_object_variable("hasAuthor"), "Creator");
    }

    #[test]
    fn test_regex_is_compiled_once_and_cached() {
        let replacement = Replacement::new(r"^has(\w+)", "$1");

        let first = replacement.regex().expect("should compile");
        let second = replacement.regex().expect("should compile");

        assert!(
            std::ptr::eq(first, second),
            "the compiled pattern should be cached, not recompiled"
        );
    }

    #[test]
    fn test_invalid_regex_reports_an_error_every_time() {
        let replacement = Replacement::new(r"([unclosed", "");

        // INFO: nothing is cached for an invalid pattern, but it keeps failing
        // in the same way instead of panicking.
        assert!(replacement.regex().is_err());
        assert!(replacement.regex().is_err());
    }

    #[test]
    fn test_replacement_equality_ignores_the_compiled_cache() {
        let uncompiled = Replacement::new(r"^has(\w+)", "$1");
        let compiled = Replacement::new(r"^has(\w+)", "$1");
        compiled.regex().expect("should compile");

        assert_eq!(uncompiled, compiled);
    }

    #[test]
    fn test_default_replacements_strip_has_prefix_camel_case() {
        let replacements = Replacements::default();

        assert_eq!(replacements.apply_object_variable("hasAuthor"), "Author");
        assert_eq!(
            replacements.apply_object_variable("hasBirthDate"),
            "Birth_Date"
        );
        // INFO: a single trailing character is not an `\w*` match for `[A-Z]\w*`,
        // but `[A-Z]` alone still matches because `\w*` may be empty.
        assert_eq!(replacements.apply_object_variable("hasX"), "X");
    }

    #[test]
    fn test_default_replacements_split_camel_case() {
        let replacements = Replacements::default();

        // INFO: lowercasing is not done here, `to_sparql_variable` does it.
        assert_eq!(
            replacements.apply_object_variable("birthDate"),
            "birth_Date"
        );
        assert_eq!(
            replacements.apply_object_variable("placeOfBirth"),
            "place_Of_Birth"
        );
        // INFO: no lowercase-to-uppercase boundary, so acronyms stay intact.
        assert_eq!(replacements.apply_object_variable("ISBN"), "ISBN");
        assert_eq!(replacements.apply_object_variable("hasISBN"), "ISBN");
    }

    #[test]
    fn test_default_replacements_strip_has_prefix_space_separated() {
        let replacements = Replacements::default();

        assert_eq!(replacements.apply_object_variable("has author"), "author");
        // NOTE: `^has (\w+)` only captures the first word, the remaining words
        // are joined by the separator collapsing of the later patterns.
        assert_eq!(
            replacements.apply_object_variable("has birth date"),
            "birth_date"
        );
    }

    #[test]
    fn test_default_replacements_strip_ed_by_suffix() {
        let replacements = Replacements::default();

        assert_eq!(replacements.apply_object_variable("authoredBy"), "author");
        assert_eq!(replacements.apply_object_variable("directedBy"), "direct");
    }

    #[test]
    fn test_default_replacements_collapse_non_word_characters() {
        let replacements = Replacements::default();

        assert_eq!(
            replacements.apply_object_variable("place of birth"),
            "place_of_birth"
        );
        assert_eq!(replacements.apply_object_variable("part-of"), "part_of");
        assert_eq!(
            replacements.apply_object_variable("under_score"),
            "under_score"
        );
        // NOTE: a run of invalid characters collapses into a single separator.
        assert_eq!(
            replacements.apply_object_variable("date  of - birth"),
            "date_of_birth"
        );
    }

    #[test]
    fn test_default_replacements_trim_leading_and_trailing_separators() {
        let replacements = Replacements::default();

        // WARNING: without the trim, the collapsing above would leave a
        // trailing "_" here.
        assert_eq!(replacements.apply_object_variable("P31/P279*"), "P31_P279");
        assert_eq!(replacements.apply_object_variable(" author "), "author");
        assert_eq!(replacements.apply_object_variable("(author)"), "author");
        // INFO: a name made up entirely of invalid characters collapses to
        // nothing, `to_sparql_variable` turns that into "var".
        assert_eq!(replacements.apply_object_variable("///"), "");
    }

    #[test]
    fn test_default_replacements_leave_unmatched_names_untouched() {
        let replacements = Replacements::default();

        assert_eq!(replacements.apply_object_variable("author"), "author");
        // INFO: "has" is only stripped as a prefix followed by a space or an
        // uppercase letter, not as a bare word or a lowercase continuation.
        assert_eq!(replacements.apply_object_variable("hasty"), "hasty");
        assert_eq!(replacements.apply_object_variable("has"), "has");
        // INFO: the "has" here is not a prefix, so only the camelCase split applies.
        assert_eq!(
            replacements.apply_object_variable("overhasAuthor"),
            "overhas_Author"
        );
    }

    #[test]
    fn test_replacements_are_applied_in_order() {
        // NOTE: the second replacement must see the output of the first.
        let replacements = Replacements {
            object_variable: vec![
                Replacement::new(r"^has(\w+)", "$1"),
                Replacement::new(r"^Author$", "creator"),
            ],
        };

        assert_eq!(replacements.apply_object_variable("hasAuthor"), "creator");
    }

    #[test]
    fn test_empty_replacements_are_a_no_op() {
        let replacements = Replacements {
            object_variable: vec![],
        };

        assert_eq!(replacements.apply_object_variable("hasAuthor"), "hasAuthor");
    }

    #[test]
    fn test_replacements_deserialize_from_yaml() {
        let yaml = r#"
            objectVariable:
              - pattern: "^has (\\w+)"
                replacement: "$1"
              - pattern: "Suffix$"
                replacement: ""
        "#;

        let replacements: Replacements = parse_yaml(yaml);

        assert_eq!(
            replacements.object_variable,
            vec![
                Replacement::new(r"^has (\w+)", "$1"),
                Replacement::new(r"Suffix$", ""),
            ]
        );
        assert_eq!(
            replacements.apply_object_variable("has authorSuffix"),
            "author"
        );
    }

    #[test]
    fn test_settings_replacements_deserialize_from_yaml() {
        let yaml = r#"
            format: {}
            completion: {}
            replacements:
              objectVariable:
                - pattern: "^is(\\w+)"
                  replacement: "$1"
        "#;

        let settings: Settings = parse_yaml(yaml);
        let replacements = settings
            .replacements
            .expect("replacements should be deserialized");

        assert_eq!(replacements.object_variable.len(), 1);
        assert_eq!(replacements.apply_object_variable("isPartOf"), "PartOf");
    }

    #[test]
    fn test_settings_default_includes_default_replacements() {
        let settings = Settings::default();

        assert_eq!(
            settings.replacements,
            Some(Replacements::default()),
            "default settings should carry the default replacements"
        );
    }

    #[test]
    fn test_settings_without_replacements_key_is_none() {
        let yaml = r#"
            format: {}
            completion: {}
        "#;

        let settings: Settings = parse_yaml(yaml);

        // WARNING: an omitted `replacements` key disables replacements entirely,
        // it does NOT fall back to `Replacements::default()`.
        assert_eq!(settings.replacements, None);
    }
}
