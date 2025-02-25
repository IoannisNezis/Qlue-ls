use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

// FIXME: This should not be exposed to Wasm API
// This is a dirty hack to get things done in QLever.
// QLever UI will implement a LSP-Client soon^TM.
#[wasm_bindgen]
pub fn determine_operation_type(text: String) -> Result<String, String> {
    let mut parser = Parser::new();
    match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        Ok(()) => {
            let tree = parser
                .parse(text.as_bytes(), None)
                .ok_or("Could not parse input")?;
            let root_node = tree.root_node();
            let mut cursor = root_node.walk();
            for op in root_node.children(&mut cursor) {
                match op.kind() {
                    "Query" => return Ok("Query".into()),
                    "Update" => return Ok("Update".into()),
                    _ => {}
                }
            }
            Ok("Unknown".into())
        }
        Err(e) => Err(format!("Could not set up parser: {}", e)),
    }
}

#[cfg(test)]
mod tests;
