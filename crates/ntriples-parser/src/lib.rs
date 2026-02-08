mod lexer;

use lexer::Token;
use logos::Logos;

pub struct Triple<'a>(pub &'a [u8], pub &'a [u8], pub &'a [u8]);

pub fn parse<'a>(input: &'a [u8]) -> Result<Vec<Triple<'a>>, ()> {
    let tokens = Token::lexer(input);
    let mut triples = Vec::new();
    let mut counter: u8 = 0;
    let mut subject: &[u8] = &[];
    let mut predicate: &[u8] = &[];
    for token in tokens {
        let token = token?;
        // NOTE: skip comments without advancing the state counter
        if matches!(token, Token::Comment) {
            continue;
        }
        match (counter, token) {
            (0, Token::Iri(bytes) | Token::BlankNode(bytes)) => {
                subject = bytes;
            }
            (1, Token::Iri(bytes)) => {
                predicate = bytes;
            }
            (2, Token::Literal(bytes) | Token::Iri(bytes) | Token::BlankNode(bytes)) => {
                triples.push(Triple(subject, predicate, bytes))
            }
            (3, Token::Dot) => {}
            _ => return Err(()),
        }
        counter = (counter + 1) % 4;
    }
    if counter != 0 {
        return Err(());
    }
    Ok(triples)
}

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn parse_comment_line() {
        let input = b"# this is a comment\n<s> <p> <o> .";
        let triples = parse(input).unwrap();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].0, b"<s>");
    }

    #[test]
    fn parse_invalid_input_returns_err() {
        let input = b"not valid ntriples!";
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_incomplete_triple_returns_err() {
        let input = b"<s> <p>";
        assert!(parse(input).is_err());
    }

    #[test]
    fn parse_blank_node_as_object() {
        let input = b"_:a <p> _:b .";
        let triples = parse(input).unwrap();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].2, b"_:b");
    }
}
