use crate::{
    Server,
    server::lsp::{
        SemanticTokensFullRequest, SemanticTokensFullResponse, SemanticTokensRangeRequest,
        capabilities::server::{SemanticTokenModifiers, SemanticTokenTypes},
        errors::LSPError,
        textdocument::Range,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::{SyntaxNode, syntax_kind::SyntaxKind};
use std::{collections::HashMap, rc::Rc};

pub const TOKEN_TYPES: [SemanticTokenTypes; 8] = [
    SemanticTokenTypes::Keyword,
    SemanticTokenTypes::Function,
    SemanticTokenTypes::Variable,
    SemanticTokenTypes::String,
    SemanticTokenTypes::Number,
    SemanticTokenTypes::Comment,
    SemanticTokenTypes::Operator,
    SemanticTokenTypes::Namespace,
];

pub(super) async fn handle_semantic_tokens_full_request(
    server_rc: Rc<Mutex<Server>>,
    request: SemanticTokensFullRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let syntax_tree = server
        .state
        .get_cached_parse_tree(&request.params.text_document.uri)?
        .tree;
    let semantic_tokens = collect_semantic_tokens(syntax_tree, None);
    let encoded_semantic_tokens = encode_semantic_tokens(semantic_tokens);
    server.send_message(SemanticTokensFullResponse::new(
        &request.base.id,
        encoded_semantic_tokens,
    ))
}

pub(super) async fn handle_semantic_tokens_range_request(
    server_rc: Rc<Mutex<Server>>,
    request: SemanticTokensRangeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let syntax_tree = server
        .state
        .get_cached_parse_tree(&request.params.text_document.uri)?
        .tree;
    let semantic_tokens = collect_semantic_tokens(syntax_tree, Some(request.params.range));
    let encoded_semantic_tokens = encode_semantic_tokens(semantic_tokens);
    server.send_message(SemanticTokensFullResponse::new(
        &request.base.id,
        encoded_semantic_tokens,
    ))
}

fn collect_semantic_tokens(
    syntax_node: SyntaxNode,
    range: Option<Range>,
) -> Vec<InternalSemanticToken> {
    let mut line: usize = 0;
    let mut char: usize = 0;
    let mut semantic_tokens = Vec::new();
    let mut next_token = syntax_node.first_token();
    while let Some(ref token) = next_token {
        let length = token.text_range().len().into();
        let (end_line, end_char) = if token.kind() == SyntaxKind::WHITESPACE {
            if let Some(offset) = token.text().rfind('\n') {
                let line_break_count = token.text().chars().filter(|char| char == &'\n').count();
                (line + line_break_count, length - offset - 1)
            } else {
                (line, char + token.text().len())
            }
        } else {
            (line, char + token.text().len())
        };
        let token_range = Range::new(line as u32, char as u32, end_line as u32, end_char as u32);
        if range
            .as_ref()
            .is_none_or(|range| token_range.overlaps(range))
            && let Some(semantic_token_type) = match token.kind() {
            // Keywords
            SyntaxKind::SELECT
            | SyntaxKind::ASK
            | SyntaxKind::DESCRIBE
            | SyntaxKind::CONSTRUCT
            | SyntaxKind::WHERE
            | SyntaxKind::PREFIX
            | SyntaxKind::BASE
            | SyntaxKind::ORDER
            | SyntaxKind::BY
            | SyntaxKind::AS
            | SyntaxKind::GROUP
            | SyntaxKind::HAVING
            | SyntaxKind::DISTINCT
            | SyntaxKind::REDUCED
            | SyntaxKind::FROM
            | SyntaxKind::NAMED
            | SyntaxKind::LIMIT
            | SyntaxKind::OFFSET
            | SyntaxKind::VALUES
            | SyntaxKind::UNDEF
            | SyntaxKind::OPTIONAL
            | SyntaxKind::UNION
            | SyntaxKind::MINUS
            | SyntaxKind::GRAPH
            | SyntaxKind::SERVICE
            | SyntaxKind::SILENT
            | SyntaxKind::BIND
            | SyntaxKind::ASC
            | SyntaxKind::DESC
            | SyntaxKind::NOT
            | SyntaxKind::IN
            | SyntaxKind::EXISTS
            | SyntaxKind::INSERT
            | SyntaxKind::DELETE
            | SyntaxKind::DATA
            | SyntaxKind::LOAD
            | SyntaxKind::CLEAR
            | SyntaxKind::DROP
            | SyntaxKind::CREATE
            | SyntaxKind::ADD
            | SyntaxKind::MOVE
            | SyntaxKind::COPY
            | SyntaxKind::WITH
            | SyntaxKind::USING
            | SyntaxKind::INTO
            | SyntaxKind::TO
            | SyntaxKind::DEFAULT
            | SyntaxKind::ALL
            | SyntaxKind::INSERT_DATA
            | SyntaxKind::DELETE_DATA
            | SyntaxKind::DELETE_WHERE
            | SyntaxKind::SEPARATOR
            | SyntaxKind::a
            | SyntaxKind::True
            | SyntaxKind::False
            | SyntaxKind::NIL => Some(SemanticTokenTypes::Keyword),

            // Built-in functions
            SyntaxKind::FILTER
            | SyntaxKind::STR
            | SyntaxKind::LANG
            | SyntaxKind::LANGMATCHES
            | SyntaxKind::DATATYPE
            | SyntaxKind::BOUND
            | SyntaxKind::IRI
            | SyntaxKind::URI
            | SyntaxKind::BNODE
            | SyntaxKind::RAND
            | SyntaxKind::ABS
            | SyntaxKind::CEIL
            | SyntaxKind::FLOOR
            | SyntaxKind::ROUND
            | SyntaxKind::CONCAT
            | SyntaxKind::STRLEN
            | SyntaxKind::UCASE
            | SyntaxKind::LCASE
            | SyntaxKind::ENCODE_FOR_URI
            | SyntaxKind::CONTAINS
            | SyntaxKind::STRSTARTS
            | SyntaxKind::STRENDS
            | SyntaxKind::STRBEFORE
            | SyntaxKind::STRAFTER
            | SyntaxKind::YEAR
            | SyntaxKind::MONTH
            | SyntaxKind::DAY
            | SyntaxKind::HOURS
            | SyntaxKind::MINUTES
            | SyntaxKind::SECONDS
            | SyntaxKind::TIMEZONE
            | SyntaxKind::TZ
            | SyntaxKind::NOW
            | SyntaxKind::UUID
            | SyntaxKind::STRUUID
            | SyntaxKind::MD5
            | SyntaxKind::SHA1
            | SyntaxKind::SHA256
            | SyntaxKind::SHA384
            | SyntaxKind::SHA512
            | SyntaxKind::COALESCE
            | SyntaxKind::IF
            | SyntaxKind::STRLANG
            | SyntaxKind::STRDT
            | SyntaxKind::sameTerm
            | SyntaxKind::isIRI
            | SyntaxKind::isURI
            | SyntaxKind::isBLANK
            | SyntaxKind::isLITERAL
            | SyntaxKind::isNUMERIC
            | SyntaxKind::REGEX
            | SyntaxKind::SUBSTR
            | SyntaxKind::REPLACE
            // Aggregates
            | SyntaxKind::COUNT
            | SyntaxKind::SUM
            | SyntaxKind::MIN
            | SyntaxKind::MAX
            | SyntaxKind::AVG
            | SyntaxKind::SAMPLE
            | SyntaxKind::GROUP_CONCAT => Some(SemanticTokenTypes::Function),

            // String literals
            SyntaxKind::STRING_LITERAL1
            | SyntaxKind::STRING_LITERAL2
            | SyntaxKind::STRING_LITERAL_LONG1
            | SyntaxKind::STRING_LITERAL_LONG2 => Some(SemanticTokenTypes::String),

            // Variables
            SyntaxKind::VAR1 | SyntaxKind::VAR2 => Some(SemanticTokenTypes::Variable),

            // Numeric literals
            SyntaxKind::INTEGER
            | SyntaxKind::DECIMAL
            | SyntaxKind::DOUBLE
            | SyntaxKind::INTEGER_POSITIVE
            | SyntaxKind::DECIMAL_POSITIVE
            | SyntaxKind::DOUBLE_POSITIVE
            | SyntaxKind::INTEGER_NEGATIVE
            | SyntaxKind::DECIMAL_NEGATIVE
            | SyntaxKind::DOUBLE_NEGATIVE => Some(SemanticTokenTypes::Number),

            // Comments
            SyntaxKind::Comment => Some(SemanticTokenTypes::Comment),

            // Operators
            SyntaxKind::DoublePipe
            | SyntaxKind::DoubleAnd
            | SyntaxKind::Equals
            | SyntaxKind::ExclamationMarkEquals
            | SyntaxKind::Less
            | SyntaxKind::More
            | SyntaxKind::LessEquals
            | SyntaxKind::MoreEquals
            | SyntaxKind::Star
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Slash
            | SyntaxKind::ExclamationMark
            | SyntaxKind::Zirkumflex
            | SyntaxKind::Pipe
            | SyntaxKind::DoubleZirkumflex => Some(SemanticTokenTypes::Operator),

            // IRIs and prefixed names
            SyntaxKind::IRIREF
            | SyntaxKind::PNAME_LN
            | SyntaxKind::PNAME_NS => Some(SemanticTokenTypes::Namespace),

            _ => None,
        } {
            semantic_tokens.push(InternalSemanticToken {
                line,
                start_char: char,
                length,
                token_type: semantic_token_type,
                token_modifier: Vec::new(),
            });
        }

        line = end_line;
        char = end_char;
        next_token = token.next_token();
    }
    semantic_tokens
}

fn encode_semantic_tokens(semantic_tokens: Vec<InternalSemanticToken>) -> Vec<u32> {
    let semant_token_id_lookup_map: HashMap<&SemanticTokenTypes, u32> = HashMap::from_iter(
        TOKEN_TYPES
            .iter()
            .enumerate()
            .map(|(idx, token_type)| (token_type, idx as u32)),
    );
    let mut result = Vec::new();
    let mut prev_line = 0;
    let mut prev_char = 0;
    for semantic_token in semantic_tokens {
        let line_delta = (semantic_token.line - prev_line) as u32;
        let char_delta = if line_delta > 0 {
            semantic_token.start_char
        } else {
            semantic_token.start_char - prev_char
        } as u32;
        let length = semantic_token.length as u32;
        result.extend_from_slice(&[
            line_delta,
            char_delta,
            length,
            *semant_token_id_lookup_map
                .get(&semantic_token.token_type)
                .expect("Every semantic token should be in the Semantic token legend."),
            0,
        ]);
        prev_line = semantic_token.line;
        prev_char = semantic_token.start_char;
    }
    result
}

/// Internal representation of a semantic token.
/// The position is absolute.
#[derive(Debug, PartialEq)]
struct InternalSemanticToken {
    line: usize,
    start_char: usize,
    length: usize,
    token_type: SemanticTokenTypes,
    token_modifier: Vec<SemanticTokenModifiers>,
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use ll_sparql_parser::parse;

    use crate::server::{
        lsp::{capabilities::server::SemanticTokenTypes, textdocument::Range},
        message_handler::semantic_tokens::{
            InternalSemanticToken, collect_semantic_tokens, encode_semantic_tokens,
        },
    };
    #[test]
    fn semantic_token_encoding() {
        let input = indoc! {
        r#"SELECT ?o WHERE {
             FILTER (LANG(?o) = "en")
           }"#
        };
        let tree = parse(input).0;
        let semantic_tokens = collect_semantic_tokens(tree, None);
        let encoding = encode_semantic_tokens(semantic_tokens);
        pretty_assertions::assert_eq!(
            encoding,
            [
                [0, 0, 6, 0, 0],
                [0, 7, 2, 2, 0],
                [0, 3, 5, 0, 0],
                [1, 2, 6, 1, 0],
                [0, 8, 4, 1, 0],
                [0, 5, 2, 2, 0],
                [0, 4, 1, 6, 0],
                [0, 2, 4, 3, 0]
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn semantic_token_computation() {
        let input = indoc! {
        r#"SELECT ?o WHERE {
             FILTER (LANG(?o) = "en")
           }"#
        };
        let tree = parse(input).0;
        let semantic_tokens = collect_semantic_tokens(tree, None);

        pretty_assertions::assert_eq!(
            semantic_tokens,
            vec![
                InternalSemanticToken {
                    line: 0,
                    start_char: 0,
                    length: 6,
                    token_type: SemanticTokenTypes::Keyword,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 0,
                    start_char: 7,
                    length: 2,
                    token_type: SemanticTokenTypes::Variable,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 0,
                    start_char: 10,
                    length: 5,
                    token_type: SemanticTokenTypes::Keyword,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 2,
                    length: 6,
                    token_type: SemanticTokenTypes::Function,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 10,
                    length: 4,
                    token_type: SemanticTokenTypes::Function,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 15,
                    length: 2,
                    token_type: SemanticTokenTypes::Variable,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 19,
                    length: 1,
                    token_type: SemanticTokenTypes::Operator,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 21,
                    length: 4,
                    token_type: SemanticTokenTypes::String,
                    token_modifier: Vec::new()
                }
            ]
        );
    }

    #[test]
    fn semantic_token_range_computation() {
        let input = indoc! {
        r#"SELECT ?o WHERE {
             FILTER (LANG(?o) = "en")
           }"#
        };
        let tree = parse(input).0;
        let semantic_tokens = collect_semantic_tokens(tree, Some(Range::new(1, 5, 1, 12)));

        pretty_assertions::assert_eq!(
            semantic_tokens,
            vec![
                InternalSemanticToken {
                    line: 1,
                    start_char: 2,
                    length: 6,
                    token_type: SemanticTokenTypes::Function,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 1,
                    start_char: 10,
                    length: 4,
                    token_type: SemanticTokenTypes::Function,
                    token_modifier: Vec::new()
                },
            ]
        );
    }

    #[test]
    fn semantic_token_computation_blank_lines() {
        let input = "\nSELECT\n\n\nFILTER";
        let tree = parse(input).0;
        let semantic_tokens = collect_semantic_tokens(tree, None);

        pretty_assertions::assert_eq!(
            semantic_tokens,
            vec![
                InternalSemanticToken {
                    line: 1,
                    start_char: 0,
                    length: 6,
                    token_type: SemanticTokenTypes::Keyword,
                    token_modifier: Vec::new()
                },
                InternalSemanticToken {
                    line: 4,
                    start_char: 0,
                    length: 6,
                    token_type: SemanticTokenTypes::Function,
                    token_modifier: Vec::new()
                }
            ]
        );
    }
}
