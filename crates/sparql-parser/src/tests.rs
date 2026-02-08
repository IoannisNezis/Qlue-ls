use crate::syntax_kind::SyntaxKind;
use logos::Logos;

fn lex_all(input: &str) -> Vec<SyntaxKind> {
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
fn tokenize_strings() {
    let tokens =
        lex_all(r#""simple string" 'other' """long\n #comment boy""" '''long\n #comment boy'''"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::STRING_LITERAL_LONG1
        ]
    )
}

#[test]
fn tokenize_insert_data() {
    let tokens = lex_all(r#"INSERT DATA { <a> <b> "'." .}"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::INSERT_DATA,
            SyntaxKind::LCurly,
            SyntaxKind::IRIREF,
            SyntaxKind::IRIREF,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::Dot,
            SyntaxKind::RCurly
        ]
    );
}

#[test]
fn tokenize_insert() {
    let input = "INSERT IN INSERT DATA INasd  in  DATA";
    let tokens = lex_all(input);

    let mut lexer = SyntaxKind::lexer(input);
    while let Some(token) = lexer.next() {
        println!("{:?} => {:?}", token, lexer.slice());
    }
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::INSERT,
            SyntaxKind::IN,
            SyntaxKind::INSERT_DATA,
            SyntaxKind::Error,
            SyntaxKind::IN,
            SyntaxKind::DATA
        ]
    );
}

#[test]
fn tokenize_blank_node_label() {
    let tokens = lex_all(r#"_:asdasdbc _:a _:_-- _:a...a _:123.345.abc"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::BLANK_NODE_LABEL
        ]
    )
}

#[test]
fn tokenize_langtag() {
    let tokens = lex_all(r#""dings"@de "foo"@a-109283"#);
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
    let tokens = lex_all(r#"delete delete               where where"#);
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::DELETE,
            SyntaxKind::DELETE_WHERE,
            SyntaxKind::WHERE,
        ]
    )
}

#[test]
fn tokenize_brack() {
    let tokens = lex_all("[ [ [");
    assert_eq!(
        tokens,
        vec![SyntaxKind::LBrack, SyntaxKind::LBrack, SyntaxKind::LBrack,]
    )
}

#[test]
fn tokenize_anon() {
    let tokens = lex_all("[] [ ] [                             ][ ?var ] [\n]");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::ANON,
            SyntaxKind::ANON,
            SyntaxKind::ANON,
            SyntaxKind::LBrack,
            SyntaxKind::VAR1,
            SyntaxKind::RBrack,
            SyntaxKind::ANON,
        ]
    )
}

#[test]
fn tokenize_a() {
    let tokens = lex_all("abc a affiliationa ada");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::Error,
            SyntaxKind::a,
            SyntaxKind::Error,
            SyntaxKind::Error,
        ]
    )
}

#[test]
fn tokenize_variables() {
    let tokens = lex_all("?var $x ?x2 ?münchen ?42 ?2· ?x ?a_b");
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
    let tokens = lex_all("42 4.2 .42 +1 -1 +1.2 -1.3 -.2 1.2e+9");
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
fn tokenize_pname_ns() {
    let tokens = lex_all("prefix: n.a: :");
    assert_eq!(
        tokens,
        vec![
            SyntaxKind::PNAME_NS,
            SyntaxKind::PNAME_NS,
            SyntaxKind::PNAME_NS,
        ]
    )
}

#[test]
fn tokenize_iris() {
    let tokens = lex_all("<simple> preix: ns:local2 ns:123 ns:%32 x....42: äöü:öäü");
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
