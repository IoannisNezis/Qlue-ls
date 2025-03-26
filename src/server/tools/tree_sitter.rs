use tree_sitter::Parser;
pub(super) fn init() -> Parser {
    let mut parser = Parser::new();
    if let Err(err) = parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        log::error!("Error while initializing parser: {}", err);
    }
    parser
}
