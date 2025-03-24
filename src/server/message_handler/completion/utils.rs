use ll_sparql_parser::ast::QueryUnit;
use sparql::results::RDFTerm;

use crate::server::{
    lsp::textdocument::{Range, TextEdit},
    Server,
};

pub(super) fn compress_rdf_term(
    server: &Server,
    query_unit: &QueryUnit,
    rdf_term: &RDFTerm,
) -> (String, Option<TextEdit>) {
    match rdf_term {
        RDFTerm::Uri { ref value } => match server.shorten_uri(value) {
            Some((prefix, uri, curie)) => {
                let prefix_decl_edit = if query_unit.prologue().as_ref().map_or(true, |prologue| {
                    prologue
                        .prefix_declarations()
                        .iter()
                        .all(|prefix_declaration| {
                            prefix_declaration
                                .prefix()
                                .map_or(false, |declared_prefix| declared_prefix != prefix)
                        })
                }) {
                    Some(TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri),
                    ))
                } else {
                    None
                };
                (curie, prefix_decl_edit)
            }
            None => (rdf_term.to_string(), None),
        },
        _ => (rdf_term.to_string(), None),
    }
}
