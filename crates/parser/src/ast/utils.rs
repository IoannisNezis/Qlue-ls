use crate::SyntaxNode;

pub(super) fn nth_ancestor(syntax: SyntaxNode, n: usize) -> Option<SyntaxNode> {
    let mut node = syntax;
    for _x in 0..n {
        node = node.parent()?;
    }
    return Some(node);
}
