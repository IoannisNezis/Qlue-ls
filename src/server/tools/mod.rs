mod tera;
mod tree_sitter;

use ::tera::Tera;
use ::tree_sitter::Parser;

pub(super) struct Tools {
    pub(super) parser: Parser,
    pub(super) tera: Tera,
}

impl Tools {
    pub(super) fn init() -> Self {
        Self {
            parser: tree_sitter::init(),
            tera: tera::init(),
        }
    }
}
