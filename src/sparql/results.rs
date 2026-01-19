use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use lazy_sparql_result_reader::sparql::RDFValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SparqlResult {
    #[allow(dead_code)]
    pub head: SparqlResultsVars,
    pub results: SparqlResultsBindings,
    #[serde(skip_deserializing)]
    pub prefixes: HashMap<String, String>,
}

#[cfg(target_arch = "wasm32")]
impl SparqlResult {
    pub fn new(vars: Vec<String>, bindings: Vec<HashMap<String, RDFTerm>>) -> Self {
        Self {
            head: SparqlResultsVars { vars },
            results: SparqlResultsBindings { bindings },
            prefixes: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SparqlResultsVars {
    #[allow(dead_code)]
    pub vars: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SparqlResultsBindings {
    pub bindings: Vec<HashMap<String, RDFTerm>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RDFTerm {
    Uri {
        value: String,
        #[serde(skip_deserializing)]
        curie: Option<String>,
    },
    Literal {
        value: String,
        #[serde(rename = "xml:lang", skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        datatype: Option<String>,
    },
    Bnode {
        value: String,
    },
}

impl RDFTerm {
    pub fn value(&self) -> &str {
        match self {
            RDFTerm::Bnode { value }
            | RDFTerm::Literal {
                value,
                lang: _,
                datatype: _,
            }
            | RDFTerm::Uri { value, curie: _ } => value,
        }
    }
}

impl Display for RDFTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RDFTerm::Uri {
                value,
                curie: _curie,
            } => write!(f, "<{}>", value),
            RDFTerm::Literal {
                value,
                lang,
                datatype,
            } => {
                if let Some(lang) = lang {
                    write!(f, "\"{}\"@{}", value, lang)
                } else if let Some(datatype) = datatype {
                    match datatype.as_str() {
                        "http://www.w3.org/2001/XMLSchema#string" => write!(f, "\"{}\"", value),
                        "http://www.w3.org/2001/XMLSchema#integer"
                        | "http://www.w3.org/2001/XMLSchema#int"
                        | "http://www.w3.org/2001/XMLSchema#decimal"
                        | "http://www.w3.org/2001/XMLSchema#double"
                        | "http://www.w3.org/2001/XMLSchema#boolean" => write!(f, "{}", value),
                        _ => write!(f, "\"{}\"^^<{}>", value, datatype),
                    }
                } else {
                    write!(f, "\"{}\"", value)
                }
            }
            RDFTerm::Bnode { value } => write!(f, "_:{}", value),
        }
    }
}

impl Into<RDFValue> for RDFTerm {
    fn into(self) -> RDFValue {
        match self {
            RDFTerm::Uri { value, curie } => RDFValue::Uri { value, curie },
            RDFTerm::Literal {
                value,
                lang,
                datatype,
            } => RDFValue::Literal {
                value,
                lang,
                datatype,
            },
            RDFTerm::Bnode { value } => RDFValue::Bnode { value },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sparql::results::RDFTerm;

    use super::SparqlResult;

    #[test]
    fn test_rdfterm_to_string() {
        // Test rendering of Uri variant
        let uri = RDFTerm::Uri {
            value: "http://example.org/resource".to_string(),
            curie: None,
        };
        assert_eq!(uri.to_string(), "<http://example.org/resource>");

        // Test rendering of Literal variant with only value
        let literal_value_only = RDFTerm::Literal {
            value: "Hello".to_string(),
            lang: None,
            datatype: None,
        };
        assert_eq!(literal_value_only.to_string(), "\"Hello\"");

        // Test rendering of Literal variant with language
        let literal_with_lang = RDFTerm::Literal {
            value: "Bonjour".to_string(),
            lang: Some("fr".to_string()),
            datatype: None,
        };
        assert_eq!(literal_with_lang.to_string(), "\"Bonjour\"@fr");

        // Test rendering of Literal variant with datatype
        let literal_with_datatype = RDFTerm::Literal {
            value: "42".to_string(),
            lang: None,
            datatype: Some("http://www.w3.org/2001/XMLSchema#integer".to_string()),
        };
        assert_eq!(literal_with_datatype.to_string(), "42");

        // Test rendering of Bnode variant
        let bnode = RDFTerm::Bnode {
            value: "b123".to_string(),
        };
        assert_eq!(bnode.to_string(), "_:b123");
    }

    #[test]
    fn deserialize() {
        let result_str = r#"{
  "head": { "vars": [ "first" , "second" ]
  } ,
  "results": { 
    "bindings": [
      {
        "first": { "type": "uri", "value": "http://example.org/book/book6"},
        "second": { "type": "literal" , "value": "test 1234" } 
      } ,
      {
        "first": { "type": "literal" , "value": "test 1234", "xml:lang": "en" } ,
        "second": { "type": "literal" , "value": "test 1234" , "datatype": "int" } 
      } ,
      {
        "first": { "type": "bnode" , "value": "dings" }
      }
    ]
  }
}"#;
        let results: SparqlResult = serde_json::from_str(result_str).unwrap();
        assert_eq!(results.head.vars, vec!["first", "second"]);
        assert!(matches!(
            results.results.bindings[0].get("first").unwrap(),
            RDFTerm::Uri { value: _, curie: _ }
        ));
        assert!(matches!(
            results.results.bindings[0].get("second").unwrap(),
            RDFTerm::Literal {
                value: _,
                lang: None,
                datatype: None
            }
        ));
        assert!(matches!(
            results.results.bindings[1].get("first").unwrap(),
            RDFTerm::Literal {
                value: _,
                lang: Some(_),
                datatype: None
            }
        ));
        assert!(matches!(
            results.results.bindings[1].get("second").unwrap(),
            RDFTerm::Literal {
                value: _,
                lang: None,
                datatype: Some(_)
            }
        ));
        assert!(matches!(
            results.results.bindings[2].get("first").unwrap(),
            RDFTerm::Bnode { value: _ }
        ));
    }
}
