use std::{collections::HashMap, fmt};

use config::{Config, ConfigError};
use serde::{de, Deserialize, Serialize};

use super::lsp::BackendService;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct BackendsSettings {
    pub backends: HashMap<String, BackendConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfiguration {
    pub service: BackendService,
    pub request_method: Option<RequestMethod>,
    pub prefix_map: HashMap<String, String>,
    #[serde(default)]
    pub default: bool,
    pub queries: HashMap<CompletionTemplate, String>,
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
}

#[derive(Debug)]
pub struct UnknownTemplateError(String);

impl fmt::Display for UnknownTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown completion query template {}", &self.0)
    }
}

impl TryFrom<String> for CompletionTemplate {
    type Error = UnknownTemplateError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "hover" => Ok(CompletionTemplate::Hover),
            "subjectCompletion" => Ok(CompletionTemplate::SubjectCompletion),
            "predicateCompletion" | "predicateCompletionContextInsensitive" => {
                Ok(CompletionTemplate::PredicateCompletionContextInsensitive)
            }
            "predicateCompletionContextSensitive" => {
                Ok(CompletionTemplate::PredicateCompletionContextSensitive)
            }
            "objectCompletion" | "objectCompletionContextInsensitive" => {
                Ok(CompletionTemplate::ObjectCompletionContextInsensitive)
            }
            "objectCompletionContextSensitive" => {
                Ok(CompletionTemplate::ObjectCompletionContextSensitive)
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    pub separate_prologue: bool,
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
            separate_prologue: false,
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
            remove_unused: Some(false),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Replacement {
    pub pattern: String,
    pub replacement: String,
}

impl Replacement {
    pub fn new(pattern: &str, replacement: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Replacements {
    pub object_variable: Vec<Replacement>,
}

impl Default for Replacements {
    fn default() -> Self {
        Self {
            object_variable: vec![
                Replacement::new(r"^has (\w+)", "$1"),
                Replacement::new(r"\s", "_"),
                Replacement::new(r"^has([A-Z]\w*)", "$1"),
                Replacement::new(r"^(\w+)edBy", "$1"),
                Replacement::new(r"([^a-zA-Z0-9_])", ""),
            ],
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
    /// Automatically add and remove prefix declarations
    pub replacements: Option<Replacements>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: FormatSettings::default(),
            completion: CompletionSettings::default(),
            backends: None,
            prefixes: Some(PrefixesSettings::default()),
            replacements: Some(Replacements::default()),
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
            service:
              name: TestBackend
              url: https://example.com/sparql
              healthCheckUrl: https://example.com/health
            requestMethod: GET
            prefixMap:
              rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
              rdfs: http://www.w3.org/2000/01/rdf-schema#
            default: false
            queries:
              subjectCompletion: SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_detail WHERE { ?qlue_ls_entity a ?type }
              predicateCompletionContextSensitive: SELECT ?qlue_ls_entity WHERE { ?s ?qlue_ls_entity ?o }
              predicateCompletionContextInsensitive: SELECT ?qlue_ls_entity WHERE { [] ?qlue_ls_entity [] }
              objectCompletionContextSensitive: SELECT ?qlue_ls_entity WHERE { ?s ?p ?qlue_ls_entity }
              objectCompletionContextInsensitive: SELECT ?qlue_ls_entity WHERE { [] [] ?qlue_ls_entity }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.service.name, "TestBackend");
        assert_eq!(config.service.url, "https://example.com/sparql");
        assert_eq!(config.default, false);
        assert_eq!(config.queries.len(), 5);
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::SubjectCompletion));
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::PredicateCompletionContextSensitive));
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::PredicateCompletionContextInsensitive));
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::ObjectCompletionContextSensitive));
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::ObjectCompletionContextInsensitive));
    }

    #[test]
    fn test_backend_configuration_queries_subset() {
        let yaml = r#"
            service:
              name: MinimalBackend
              url: https://example.com/sparql
            prefixMap: {}
            queries:
              subjectCompletion: SELECT ?qlue_ls_entity WHERE { ?qlue_ls_entity ?p ?o }
              objectCompletionContextInsensitive: SELECT ?qlue_ls_entity WHERE { ?s ?p ?qlue_ls_entity }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.queries.len(), 2);
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::SubjectCompletion));
        assert!(config
            .queries
            .contains_key(&CompletionTemplate::ObjectCompletionContextInsensitive));
        assert!(!config
            .queries
            .contains_key(&CompletionTemplate::PredicateCompletionContextSensitive));
    }

    #[test]
    fn test_backend_configuration_rejects_invalid_query_key() {
        // This test ensures that invalid query keys are rejected
        let yaml = r#"
            service:
              name: TestBackend
              url: https://example.com/sparql
            prefixMap: {}
            queries:
              invalidQueryType: SELECT ?qlue_ls_entity WHERE { ?s ?p ?o }
              subjectCompletion: SELECT ?qlue_ls_entity WHERE { ?qlue_ls_entity ?p ?o }
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
            service:
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
                SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_detail
                WHERE {
                  ?qlue_ls_entity rdfs:label ?qlue_ls_label .
                  OPTIONAL { ?qlue_ls_entity schema:description ?qlue_ls_detail }
                  FILTER(LANG(?qlue_ls_label) = "en")
                }
                LIMIT 100
              predicateCompletionContextSensitive: |
                SELECT ?qlue_ls_entity WHERE {
                  ?s ?qlue_ls_entity ?o
                }
              objectCompletionContextInsensitive: SELECT ?qlue_ls_entity WHERE { [] [] ?qlue_ls_entity }
        "#;

        let config: BackendConfiguration = parse_yaml(yaml);

        assert_eq!(config.service.name, "WikidataBackend");
        assert_eq!(config.service.url, "https://query.wikidata.org/sparql");
        assert_eq!(config.default, false);
        assert_eq!(config.prefix_map.len(), 3);
        assert_eq!(config.queries.len(), 3);

        // Verify multiline query was parsed correctly
        let subject_query = config
            .queries
            .get(&CompletionTemplate::SubjectCompletion)
            .unwrap();
        assert!(subject_query.contains("SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_detail"));
        assert!(subject_query.contains("FILTER(LANG(?qlue_ls_label) = \"en\")"));
    }

    #[test]
    fn test_backends_settings_multiple_backends() {
        let yaml = r#"
            backends:
              wikidata:
                service:
                  name: Wikidata
                  url: https://query.wikidata.org/sparql
                prefixMap:
                  wd: http://www.wikidata.org/entity/
                queries:
                  subjectCompletion: SELECT ?qlue_ls_entity WHERE { ?qlue_ls_entity ?p ?o }
              dbpedia:
                service:
                  name: DBpedia
                  url: https://dbpedia.org/sparql
                prefixMap:
                  dbo: http://dbpedia.org/ontology/
                default: true
                queries:
                  objectCompletionContextSensitive: SELECT ?qlue_ls_entity WHERE { ?s ?p ?qlue_ls_entity }
        "#;

        let settings: BackendsSettings = parse_yaml(yaml);

        assert_eq!(settings.backends.len(), 2);
        assert!(settings.backends.contains_key("wikidata"));
        assert!(settings.backends.contains_key("dbpedia"));

        let wikidata = settings.backends.get("wikidata").unwrap();
        assert_eq!(wikidata.service.name, "Wikidata");
        assert_eq!(wikidata.queries.len(), 1);

        let dbpedia = settings.backends.get("dbpedia").unwrap();
        assert_eq!(dbpedia.service.name, "DBpedia");
        assert_eq!(dbpedia.default, true);
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
                  service:
                    name: Wikidata
                    url: https://query.wikidata.org/sparql
                    healthCheckUrl: https://query.wikidata.org/
                  prefixMap:
                    wd: http://www.wikidata.org/entity/
                    wdt: http://www.wikidata.org/prop/direct/
                  default: true
                  queries:
                    subjectCompletion: SELECT ?qlue_ls_entity WHERE { ?qlue_ls_entity ?p ?o }
                    predicateCompletionContextSensitive: SELECT ?qlue_ls_entity WHERE { ?s ?qlue_ls_entity ?o }
            prefixes:
              addMissing: true
              removeUnused: false
        "#;

        let settings: Settings = parse_yaml(yaml);

        assert_eq!(settings.format.align_predicates, true);
        assert_eq!(settings.completion.timeout_ms, 5000);
        assert!(settings.backends.is_some());

        let backends = settings.backends.unwrap();
        assert_eq!(backends.backends.len(), 1);

        let wikidata = backends.backends.get("wikidata").unwrap();
        assert_eq!(wikidata.service.name, "Wikidata");
        assert_eq!(wikidata.default, true);
        assert_eq!(wikidata.queries.len(), 2);
    }
}
