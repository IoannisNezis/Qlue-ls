use crate::{parse_query, syntax_kind::SyntaxKind};

#[test]
fn parse_tokens_after() {
    let input = "SELECT * WHERE {} hay";
    let root = parse_query(input).0;
    assert!(matches!(
        root.last_token().unwrap().kind(),
        SyntaxKind::Error
    ));
}

#[test]
fn parse_tokens_before() {
    let input = "dings;\nCLEAR <GraphRef>;\n";
    let _root = parse_query(input).0;
}
