use logos::Logos;

#[derive(Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\n\f]+", utf8 = false)] // Ignore this regex pattern between tokens
pub enum Token<'a> {
    #[regex(r"#[^\n]*", allow_greedy = true)]
    Comment,
    #[token(".")]
    Dot,
    #[regex(r#"<[^<>\"{}|^`\\\u{00}-\u{20}]*>"#)]
    Iri(&'a [u8]),
    #[regex(r"_:(?:(?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}_])|(?:[0-9]))(?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}_0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}\.-]*[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}_0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}-])?")]
    BlankNode(&'a [u8]),
    #[regex(r#""([^\u{5C}\u{A}\u{D}"]|\\[tbnrf"'\\]|\\u[0-9A-Fa-f]{4}|\\U[0-9A-Fa-f]{8})*"(\^\^<[^<>\"{}|^`\\\u{00}-\u{20}]*>|@[a-zA-Z]+(-[a-zA-Z0-9]+)*)?"#)]
    // NOTE: unquoted numeric literals (integer, decimal, double) are not part of the
    // N-Triples spec but QLever responds with them, so we accept them here.
    #[regex(r"[+-]?[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?")]
    #[regex(r"[+-]?[0-9]+([eE][+-]?[0-9]+)?")]
    Literal(&'a [u8]),
}

#[cfg(test)]
mod test {
    use super::Token;
    use logos::Logos;

    #[test]
    fn tokenize_triple_1() {
        let tokens = Token::lexer(br#"_:a <iri> "strings"@en"#);
        assert_eq!(
            tokens.into_iter().map(|token| token).collect::<Vec<_>>(),
            vec![
                Ok(Token::BlankNode(b"_:a")),
                Ok(Token::Iri(b"<iri>")),
                Ok(Token::Literal(br#""strings"@en"#))
            ]
        );
    }

    #[test]
    fn tokenize_triple_2() {
        let tokens = Token::lexer(br#"<test> <http://www.w3.org/2001/XMLSchema#test> 8.0 ."#);
        assert_eq!(
            tokens.into_iter().map(|token| token).collect::<Vec<_>>(),
            vec![
                Ok(Token::Iri(b"<test>")),
                Ok(Token::Iri(b"<http://www.w3.org/2001/XMLSchema#test>")),
                Ok(Token::Literal(br#"8.0"#)),
                Ok(Token::Dot)
            ]
        );
    }

    #[test]
    fn tokenize_blanknode() {
        let tokens = Token::lexer(br#"_:a _:n1 _:asda"#);
        assert_eq!(
            tokens.into_iter().map(|token| token).collect::<Vec<_>>(),
            vec![
                Ok(Token::BlankNode(b"_:a")),
                Ok(Token::BlankNode(b"_:n1")),
                Ok(Token::BlankNode(b"_:asda"))
            ]
        );
    }

    #[test]
    fn tokenize_iri() {
        let tokens = Token::lexer(
            br#"<asdasdsa> <http://www.wikidata.org/prop/> <http://www.wikidata.org/prop/statement/value-normalized/>"#,
        );
        assert_eq!(
            tokens.into_iter().map(|token| token).collect::<Vec<_>>(),
            vec![
                Ok(Token::Iri(b"<asdasdsa>")),
                Ok(Token::Iri(b"<http://www.wikidata.org/prop/>")),
                Ok(Token::Iri(
                    b"<http://www.wikidata.org/prop/statement/value-normalized/>"
                ))
            ]
        );
    }

    #[test]
    fn tokenize_literal() {
        let tokens = Token::lexer(br#""simple string" "hello"@en "x"^^<asdadasd> 8.0"#);
        assert_eq!(
            tokens.into_iter().map(|token| token).collect::<Vec<_>>(),
            vec![
                Ok(Token::Literal(br#""simple string""#)),
                Ok(Token::Literal(br#""hello"@en"#)),
                Ok(Token::Literal(br#""x"^^<asdadasd>"#)),
                Ok(Token::Literal(b"8.0"))
            ]
        );
    }

    #[test]
    fn tokenize_blanknode_with_inner_dot() {
        let tokens = Token::lexer(b"_:a.b");
        assert_eq!(
            tokens.into_iter().collect::<Vec<_>>(),
            vec![Ok(Token::BlankNode(b"_:a.b"))]
        );
    }

    #[test]
    fn tokenize_blanknode_trailing_dot() {
        // NOTE: trailing dot must not be consumed as part of the blank node label
        let tokens = Token::lexer(b"_:a.");
        assert_eq!(
            tokens.into_iter().collect::<Vec<_>>(),
            vec![Ok(Token::BlankNode(b"_:a")), Ok(Token::Dot)]
        );
    }

    // BUG 5: apostrophes in strings are incorrectly rejected
    #[test]
    fn tokenize_literal_with_apostrophe() {
        let tokens = Token::lexer(br#""it's a test""#);
        assert_eq!(
            tokens.into_iter().collect::<Vec<_>>(),
            vec![Ok(Token::Literal(br#""it's a test""#))]
        );
    }

    // BUG 6: escape sequences in strings are not supported
    #[test]
    fn tokenize_literal_with_escapes() {
        let tokens = Token::lexer(br#""line1\nline2" "quote\"inside" "back\\slash""#);
        assert_eq!(
            tokens.into_iter().collect::<Vec<_>>(),
            vec![
                Ok(Token::Literal(br#""line1\nline2""#)),
                Ok(Token::Literal(br#""quote\"inside""#)),
                Ok(Token::Literal(br#""back\\slash""#)),
            ]
        );
    }

    // BUG 7: only @en is accepted as a language tag
    #[test]
    fn tokenize_literal_with_language_tags() {
        let tokens = Token::lexer(br#""hallo"@de "bonjour"@fr "color"@en-US"#);
        assert_eq!(
            tokens.into_iter().collect::<Vec<_>>(),
            vec![
                Ok(Token::Literal(br#""hallo"@de"#)),
                Ok(Token::Literal(br#""bonjour"@fr"#)),
                Ok(Token::Literal(br#""color"@en-US"#)),
            ]
        );
    }
}
