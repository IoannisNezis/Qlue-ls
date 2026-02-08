use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(test, derive(Deserialize))]
pub struct SparqlResult {
    pub head: Head,
    pub results: Bindings,
    pub meta: Meta,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(rename_all = "kebab-case")]
pub struct Meta {
    // This gives max query time of (2**32-1)/1000/24 = 12810238940076 days
    pub query_time_ms: u64,
    pub result_size_total: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Bindings {
    pub bindings: Vec<Binding>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Header {
    pub head: Head,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Head {
    pub vars: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Binding(pub HashMap<String, RDFValue>);

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RDFValue {
    Uri {
        value: String,
        // NOTE: custom field used by Qlue-ls.
        curie: Option<String>,
    },
    // NOTE: Virtuoso responds with these types.
    #[serde(alias = "typed-literal")]
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
