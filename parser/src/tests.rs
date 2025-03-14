use logos::Logos;

use crate::syntax_kind::SyntaxKind;

fn tokenize(input: &str) -> Vec<SyntaxKind> {
    let mut token_kinds = Vec::new();
    let mut lexer = SyntaxKind::lexer(input);
    while let Some(result) = lexer.next() {
        // println!("kind: {:?}", result.unwrap_or(SyntaxKind::Error));
        // println!("text: {:?}", lexer.slice().to_string());
        match result {
            Ok(kind) if !kind.is_trivia() => token_kinds.push(kind),
            Err(_) => token_kinds.push(SyntaxKind::Error),
            _ => {}
        }
    }
    return token_kinds;
}

#[test]
fn tokenize_variables() {
    let tokens = tokenize("?var $x ?x2 ?münchen ?42 ?2· ?x");
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
