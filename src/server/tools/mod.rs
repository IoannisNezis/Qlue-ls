mod curies;
mod tera;
mod tree_sitter;

use ::curies::Converter;
use ::tera::Tera;
use ::tree_sitter::Parser;

pub(super) struct Tools {
    pub(super) uri_converter: Converter,
    pub(super) parser: Parser,
    pub(super) tera: Tera,
}

impl Tools {
    pub(super) fn init() -> Self {
        Self {
            uri_converter: curies::init(),
            parser: tree_sitter::init(),
            tera: tera::init(),
        }
    }
}
