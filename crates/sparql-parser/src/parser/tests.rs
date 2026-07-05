use crate::{parse_query, syntax_kind::SyntaxKind};

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
    let input = "dings;\nCLEAR <GraphRef>;\n";
    let _root = parse_query(input).0;
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
