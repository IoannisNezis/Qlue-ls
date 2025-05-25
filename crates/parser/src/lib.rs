pub mod ast;
mod parser;
mod rules;
pub mod syntax_kind;
mod syntax_node;
mod utils;

use parser::{guess_operation_type, TopEntryPoint};
pub use utils::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Object, Reflect};
#[cfg(target_arch = "wasm32")]
use rowan::TextSize;
use syntax_kind::SyntaxKind;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub use syntax_node::*;

pub fn parse_query(input: &str) -> SyntaxNode {
    SyntaxNode::new_root(parser::parse_text(input, parser::TopEntryPoint::QueryUnit))
}

pub fn parse_update(input: &str) -> SyntaxNode {
    SyntaxNode::new_root(parser::parse_text(input, parser::TopEntryPoint::UpdateUnit))
}

pub fn parse(input: &str) -> SyntaxNode {
    match guess_operation_type(input) {
        Some(TopEntryPoint::QueryUnit) | None => parse_query(input),
        Some(TopEntryPoint::UpdateUnit) => parse_update(input),
    }
}

#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_parse_tree(input: &str, offset: u32) -> JsValue {
    use parser::TopEntryPoint;

    let root = SyntaxNode::new_root(parser::parse_text(input, TopEntryPoint::QueryUnit));
    build_js_tree(&root, TextSize::new(offset))
}

#[cfg(target_arch = "wasm32")]
fn build_js_tree(node: &SyntaxNode, offset: TextSize) -> JsValue {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("kind"),
        &JsValue::from_str(&format!("{:?}", node.kind())),
    )
    .unwrap();
    Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("node")).unwrap();
    Reflect::set(
        &obj,
        &JsValue::from_str("active"),
        &JsValue::from_bool(node.text_range().contains(offset)),
    )
    .unwrap();

    let children = Array::from_iter(node.children_with_tokens().filter_map(|child| match child {
        rowan::NodeOrToken::Node(node) => Some(build_js_tree(&node, offset)),
        rowan::NodeOrToken::Token(token) => {
            let token_obj = Object::new();
            Reflect::set(
                &token_obj,
                &JsValue::from_str("kind"),
                &JsValue::from_str(&format!("{:?}", token.kind())),
            )
            .unwrap();
            Reflect::set(
                &token_obj,
                &JsValue::from_str("type"),
                &JsValue::from_str("token"),
            )
            .unwrap();
            Reflect::set(
                &token_obj,
                &JsValue::from_str("text"),
                &JsValue::from_str(token.text()),
            )
            .unwrap();
            Reflect::set(
                &token_obj,
                &JsValue::from_str("active"),
                &JsValue::from_bool(token.text_range().contains(offset)),
            )
            .unwrap();
            Some(token_obj.into())
        }
    }));
    Reflect::set(&obj, &JsValue::from_str("children"), &children.into()).unwrap();
    obj.into()
}
