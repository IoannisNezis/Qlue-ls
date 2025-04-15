use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SparqlResult {
    pub head: SparqlResultsVars,
    pub results: SparqlResultsBindings,
}

#[derive(Debug, Deserialize)]
pub struct SparqlResultsVars {
    pub vars: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SparqlResultsBindings {
    pub bindings: Vec<HashMap<String, RDFTerm>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RDFTerm {
    Uri {
        value: String,
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
impl Display for RDFTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RDFTerm::Uri { value } => write!(f, "<{}>", value),
            RDFTerm::Literal {
                value,
                lang,
                datatype,
            } => match (lang.as_ref(), datatype.as_ref()) {
                (None, None) => write!(f, "\"{}\"", value),
                (None, Some(_dt)) => write!(f, "\"{}\"", value),
                (Some(lang_tag), None) => write!(f, "\"{}\"@{}", value, lang_tag),
                (Some(_), Some(_)) => {
                    panic!("No RDFTERm should have a language tag and a datatype")
                }
            },
            RDFTerm::Bnode { value } => write!(f, "_:{}", value),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::results::RDFTerm;

    use super::SparqlResult;

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
        let results: SparqlResult = serde_json::from_str(&result_str).unwrap();
        assert_eq!(results.head.vars, vec!["first", "second"]);
        assert!(matches!(
            results.results.bindings[0].get("first").unwrap(),
            RDFTerm::Uri { value: _ }
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
