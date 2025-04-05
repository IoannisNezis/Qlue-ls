use crate::{parse_query, syntax_kind::SyntaxKind};

#[test]
fn parse_tokens_after() {
    let input = "SELECT * WHERE {} hay";
    let root = parse_query(input);
    assert!(matches!(
        root.last_token().unwrap().kind(),
        SyntaxKind::Error
    ));
}
