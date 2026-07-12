use crate::{parse_query, parse_update, syntax_kind::SyntaxKind};

#[test]
fn parse_tokens_after() {
    let input = "SELECT * WHERE {} hay";
    let root = parse_query(input).0;
    println!("{:#?}", root);
    assert!(matches!(
        root.last_token().unwrap().kind(),
        SyntaxKind::Error
    ));
}

#[test]
fn parse_tokens_before() {
    // NOTE: unexpected leading tokens must not produce an empty tree,
    // the entry point always yields a lossless UpdateUnit node
    let input = "dings\nCLEAR <GraphRef>;";
    let root = parse_update(input).0;
    assert_eq!(root.kind(), SyntaxKind::UpdateUnit);
    assert_eq!(root.text().to_string(), input);
}

#[test]
fn recover_select_clause_before_where() {
    // NOTE: the broken SelectClause must not swallow the WHERE token,
    // the WhereClause should still parse completely
    let input = "SELECT WHERE { ?a ?b ?c }";
    let root = parse_query(input).0;
    println!("{:#?}", root);
    let where_clause = root
        .descendants()
        .find(|node| node.kind() == SyntaxKind::WhereClause)
        .expect("WhereClause should exist");
    let vars: Vec<_> = where_clause
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::Var)
        .collect();
    assert_eq!(vars.len(), 3);
}
