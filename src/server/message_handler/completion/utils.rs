use ll_sparql_parser::{syntax_kind::SyntaxKind, SyntaxToken};

pub(super) fn match_ancestors(token: &SyntaxToken, ancestors: &[SyntaxKind]) -> bool {
    token
        .parent_ancestors()
        .zip(ancestors.iter())
        .take_while(|(ancestor, kind)| ancestor.kind() == **kind)
        .count()
        == ancestors.len()
}
