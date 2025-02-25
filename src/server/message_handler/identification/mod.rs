use tree_sitter::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn determine_operation_type(text: String) -> Result<String, String> {
    let mut parser = Parser::new();
    match parser.set_language(&tree_sitter_sparql::LANGUAGE.into()) {
        Ok(()) => {
            let tree = parser
                .parse(text.as_bytes(), None)
                .expect("Could not parse input");
            let root_node = tree.root_node();
            let mut cursor = root_node.walk();
            for op in root_node.children(&mut cursor) {
                match op.kind() {
                    "SelectQuery" | "ConstructQuery" | "DescribeQuery" | "AskQuery" => {
                        return Ok("Query".into())
                    }
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
