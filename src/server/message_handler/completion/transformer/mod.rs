mod object_suffix;
mod semicolon;

pub use object_suffix::ObjectSuffixTransformer;
pub use semicolon::SemicolonTransformer;

use crate::server::lsp::CompletionList;

/// A transformer that modifies completion items after they are generated.
///
/// Transformers are applied to completion items based on the completion context.
/// Use `try_from_env` on each transformer to check if it applies and create it.
pub trait CompletionTransformer {
    /// Transform a completion item in place.
    fn transform(&self, list: &mut CompletionList);
}
