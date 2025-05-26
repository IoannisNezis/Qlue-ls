use ll_sparql_parser::{guess_operation_type, TopEntryPoint};
use wasm_bindgen::prelude::wasm_bindgen;

// FIXME: This should not be exposed to Wasm API
// This is a dirty hack to get things done in QLever.
// QLever UI will implement a LSP-Client soon^TM.
#[wasm_bindgen]
pub fn determine_operation_type(input: String) -> String {
    match guess_operation_type(&input) {
        Some(TopEntryPoint::QueryUnit) => "Query",
        Some(TopEntryPoint::UpdateUnit) => "Update",
        None => "Unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests;
