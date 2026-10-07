//! Conversion between full URIs and compact URIs (CURIEs) using a backend's prefix map.
//!
//! This replaces the `curies` crate, which unconditionally depends on `reqwest`
//! (and `sophia`) only to fetch prefix maps from the web — something qlue-ls never
//! does. Only the small subset of its API that the server needs is implemented here.
//!
//! # Key Types
//!
//! - [`Converter`]: Prefix lookup, longest-match URI lookup and URI compression
//! - [`Record`]: A prefix and the URI prefix (namespace) it stands for

use std::collections::{BTreeSet, HashMap};

/// A prefix and the URI prefix (namespace) it expands to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    pub(crate) prefix: String,
    pub(crate) uri_prefix: String,
}

#[derive(Debug, Default)]
pub(crate) struct Converter {
    records: Vec<Record>,
    /// NOTE: Maps every prefix, including synonyms, to an index into `records`.
    by_prefix: HashMap<String, usize>,
    /// NOTE: Maps every URI prefix to an index into `records`.
    by_uri_prefix: HashMap<String, usize>,
    /// NOTE: Distinct byte lengths of all URI prefixes, used for longest-prefix
    ///       lookup without scanning every record.
    uri_prefix_lengths: BTreeSet<usize>,
}

impl Converter {
    /// Builds a converter from a `prefix -> uri_prefix` map.
    ///
    /// When several prefixes share a URI prefix, the alphabetically first one becomes
    /// the primary prefix (used for compression) and the others become synonyms.
    pub(crate) fn from_prefix_map(map: &HashMap<String, String>) -> Self {
        let mut converter = Self::default();
        // INFO: Sorting makes the choice of primary prefix deterministic.
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort();
        for (prefix, uri_prefix) in entries {
            converter.add_prefix(prefix, uri_prefix);
        }
        converter
    }

    fn add_prefix(&mut self, prefix: &str, uri_prefix: &str) {
        if let Some(&index) = self.by_uri_prefix.get(uri_prefix) {
            tracing::warn!(
                "URI prefix \"{uri_prefix}\" is already registered to prefix \"{}\", adding \"{prefix}\" as a synonym",
                self.records[index].prefix
            );
            self.by_prefix.insert(prefix.to_string(), index);
            return;
        }
        let index = self.records.len();
        self.records.push(Record {
            prefix: prefix.to_string(),
            uri_prefix: uri_prefix.to_string(),
        });
        self.by_prefix.insert(prefix.to_string(), index);
        self.by_uri_prefix.insert(uri_prefix.to_string(), index);
        self.uri_prefix_lengths.insert(uri_prefix.len());
    }

    /// Finds the record for a prefix (or one of its synonyms).
    pub(crate) fn find_by_prefix(&self, prefix: &str) -> Option<&Record> {
        self.by_prefix
            .get(prefix)
            .map(|&index| &self.records[index])
    }

    /// Finds the record whose URI prefix is the longest prefix of `uri`.
    pub(crate) fn find_by_uri(&self, uri: &str) -> Option<&Record> {
        self.uri_prefix_lengths
            .iter()
            .rev()
            .filter(|&&len| uri.is_char_boundary(len))
            .find_map(|&len| self.by_uri_prefix.get(&uri[..len]))
            .map(|&index| &self.records[index])
    }

    /// Compresses `uri` into a CURIE (`prefix:local`) using the longest matching URI prefix.
    pub(crate) fn compress(&self, uri: &str) -> Option<String> {
        let record = self.find_by_uri(uri)?;
        Some(format!(
            "{}:{}",
            record.prefix,
            &uri[record.uri_prefix.len()..]
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn converter(entries: &[(&str, &str)]) -> Converter {
        Converter::from_prefix_map(
            &entries
                .iter()
                .map(|(prefix, uri_prefix)| (prefix.to_string(), uri_prefix.to_string()))
                .collect(),
        )
    }

    #[test]
    fn compress_uses_longest_uri_prefix() {
        let converter = converter(&[
            ("wd", "http://www.wikidata.org/entity/"),
            ("wds", "http://www.wikidata.org/entity/statement/"),
        ]);
        assert_eq!(
            converter.compress("http://www.wikidata.org/entity/Q42"),
            Some("wd:Q42".to_string())
        );
        assert_eq!(
            converter.compress("http://www.wikidata.org/entity/statement/Q42-abc"),
            Some("wds:Q42-abc".to_string())
        );
        assert_eq!(converter.compress("http://example.org/foo"), None);
    }

    #[test]
    fn find_by_uri_ignores_non_char_boundaries() {
        let converter = converter(&[("ex", "http://example.org/")]);
        assert_eq!(converter.find_by_uri("http://ä"), None);
        assert_eq!(
            converter.compress("http://example.org/ä"),
            Some("ex:ä".to_string())
        );
    }

    #[test]
    fn duplicate_uri_prefix_becomes_synonym() {
        let converter = converter(&[
            ("schema", "https://schema.org/"),
            ("sdo", "https://schema.org/"),
        ]);
        let record = converter.find_by_prefix("sdo").unwrap();
        assert_eq!(record.prefix, "schema");
        assert_eq!(record.uri_prefix, "https://schema.org/");
        assert_eq!(converter.find_by_prefix("schema"), Some(record));
        assert_eq!(
            converter.compress("https://schema.org/name"),
            Some("schema:name".to_string())
        );
    }

    #[test]
    fn find_by_prefix_unknown() {
        let converter = converter(&[("ex", "http://example.org/")]);
        assert_eq!(converter.find_by_prefix("foo"), None);
    }
}
