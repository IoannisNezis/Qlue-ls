use crate::syntax_kind::SyntaxKind;
use logos::Logos;

fn tokenize(input: &str) -> Vec<SyntaxKind> {
    let mut token_kinds = Vec::new();
    let lexer = SyntaxKind::lexer(input);
    for result in lexer {
        match result {
            Ok(kind) if !kind.is_trivia() => token_kinds.push(kind),
            Err(_) => token_kinds.push(SyntaxKind::Error),
            _ => {}
        }
    }
    return token_kinds;
}

#[test]
fn tokenize_blank_node_label() {
    let tokens = tokenize(r#"_:asdasdbc _:_-- _:123.345.abc"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL
        ]
    )
}

#[test]
fn tokenize_langtag() {
    let tokens = tokenize(r#""dings"@de "foo"@a-109283"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LANGTAG,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LANGTAG
        ]
    )
}

#[test]
fn tokenize_delete_where() {
    let tokens = tokenize(r#"delete where where"#);
    assert_eq!(tokens, vec![SyntaxKind::DELETE_WHERE, SyntaxKind::WHERE,])
}

#[test]
fn tokenize_brack() {
    let tokens = tokenize("[] [ ] [ ?var ] ");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::ANON,
            SyntaxKind::ANON,
            SyntaxKind::LBrack,
            SyntaxKind::VAR1,
            SyntaxKind::RBrack
        ]
    )
}

#[test]
fn tokenize_a() {
    let tokens = tokenize("abc a affiliation");
    assert_eq!(
        tokens,
        vec![SyntaxKind::Error, SyntaxKind::a, SyntaxKind::Error,]
    )
}

#[test]
fn tokenize_variables() {
    let tokens = tokenize("?var $x ?x2 ?münchen ?42 ?2· ?x ?a_b");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::VAR1,
            SyntaxKind::VAR1,
            SyntaxKind::VAR1,
            SyntaxKind::VAR1,
            SyntaxKind::VAR1,
            SyntaxKind::VAR1,
        ]
    )
}

#[test]
fn tokenize_numbers() {
    let tokens = tokenize("42 4.2 .42 +1 -1 +1.2 -1.3 -.2 1.2e+9");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::INTEGER,
            SyntaxKind::DECIMAL,
            SyntaxKind::DECIMAL,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE,
        ]
    )
}

#[test]
fn tokenize_iris() {
    let tokens = tokenize("<simple> prefix: ns:local2 ns:123 ns:%32 x....42: äöü:öäü");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::PNAME_LN,
            SyntaxKind::PNAME_LN,
            SyntaxKind::PNAME_LN,
            SyntaxKind::PNAME_NS,
            SyntaxKind::PNAME_LN,
        ]
    )
}
