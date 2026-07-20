use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    SyntaxNode,
    ast::{AstNode, Prologue},
    syntax_kind::SyntaxKind,
};

use crate::server::{
    Server,
    lsp::{
        FoldingRange, FoldingRangeKind, FoldingRangeRequest, FoldingRangeResponse,
        errors::LSPError,
        textdocument::{Position, Range},
    },
};

#[tracing::instrument(skip_all, fields(id = %request.get_id(), uri = %request.get_document_uri()))]
pub(super) async fn handle_folding_range_request(
    server_rc: Rc<Mutex<Server>>,
    request: FoldingRangeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server.state.get_document(request.get_document_uri())?;
    let tree = server
        .state
        .get_cached_parse_tree(request.get_document_uri())?
        .tree;
    let result = compute_folding_ranges(&tree, &document.text);
    let mut response = FoldingRangeResponse::new(request.get_id());
    response.set_result(result);
    server.send_message(response)
}

/// Compute all folding ranges for a parsed document:
/// the prologue (as an imports region) and every curly-brace delimited group graph pattern.
fn compute_folding_ranges(tree: &SyntaxNode, text: &str) -> Vec<FoldingRange> {
    let mut result = vec![];
    if let Some(prologue) = tree
        .first_child()
        .and_then(|child| child.first_child().and_then(Prologue::cast))
    {
        let range = Range::from_byte_offset_range(prologue.syntax().text_range(), text).unwrap();
        result.push(FoldingRange {
            start_line: range.start.line,
            end_line: range.end.line,
            start_character: None,
            end_character: None,
            kind: Some(FoldingRangeKind::Imports),
            collapsed_text: Some(prologue.text()),
        });
    }
    // Fold any block delimited by curly braces
    result.extend(
        tree.descendants()
            .filter(|node| node.kind() == SyntaxKind::GroupGraphPattern)
            .filter_map(|node| {
                let open_brace = node
                    .first_child_or_token()
                    .filter(|token| token.kind() == SyntaxKind::LCurly)?
                    .into_token()?;
                let close_brace = node
                    .last_child_or_token()
                    .filter(|token| token.kind() == SyntaxKind::RCurly)?
                    .into_token()?;
                let open_position =
                    Position::from_byte_index(open_brace.text_range().start(), text)?;
                let close_position =
                    Position::from_byte_index(close_brace.text_range().start(), text)?;
                // NOTE: A folding range collapses `start_line..=end_line`, so end the fold on
                // the line before the closing brace to keep the `}` visible. `checked_sub`
                // also drops single-line blocks, which have nothing to fold.
                let end_line = close_position.line.checked_sub(1)?;
                // INFO: Skip degenerate ranges (e.g. empty two-line blocks) that would fold nothing.
                if end_line <= open_position.line {
                    return None;
                }
                Some(FoldingRange {
                    start_line: open_position.line,
                    end_line,
                    start_character: None,
                    end_character: None,
                    kind: None,
                    collapsed_text: None,
                })
            }),
    );
    result
}

#[cfg(test)]
mod test {
    use ll_sparql_parser::parse;

    use super::compute_folding_ranges;
    use crate::server::lsp::FoldingRangeKind;

    /// Returns only the brace-delimited (GGP) folding ranges as `(start_line, end_line)` tuples.
    fn ggp_folds(text: &str) -> Vec<(u32, u32)> {
        let (tree, _) = parse(text);
        compute_folding_ranges(&tree, text)
            .into_iter()
            .filter(|range| range.kind.is_none())
            .map(|range| (range.start_line, range.end_line))
            .collect()
    }

    #[test]
    fn single_group_graph_pattern() {
        let text = "SELECT * {\n  ?s ?p ?o .\n}";
        // Fold ends on the line before the closing brace so the `}` stays visible.
        assert_eq!(ggp_folds(text), vec![(0, 1)]);
    }

    #[test]
    fn nested_group_graph_patterns() {
        let text = "SELECT * {\n  ?s ?p ?o .\n  {\n    ?a ?b ?c .\n  }\n}";
        let mut folds = ggp_folds(text);
        folds.sort();
        assert_eq!(folds, vec![(0, 4), (2, 3)]);
    }

    #[test]
    fn optional_group_graph_pattern() {
        let text = "SELECT * {\n  ?s ?p ?o .\n  OPTIONAL {\n    ?a ?b ?c .\n  }\n}";
        let mut folds = ggp_folds(text);
        folds.sort();
        assert_eq!(folds, vec![(0, 4), (2, 3)]);
    }

    #[test]
    fn union_group_graph_patterns() {
        let text = "SELECT * {\n  {\n    ?s ?p ?o .\n  }\n  UNION\n  {\n    ?a ?b ?c .\n  }\n}";
        let mut folds = ggp_folds(text);
        folds.sort();
        assert_eq!(folds, vec![(0, 7), (1, 2), (5, 6)]);
    }

    #[test]
    fn single_line_group_graph_pattern_is_not_folded() {
        // Opening and closing brace share a line, so there is nothing to fold.
        let text = "SELECT * { ?s ?p ?o . }";
        assert!(ggp_folds(text).is_empty());
    }

    #[test]
    fn empty_group_graph_pattern_does_not_fold_closing_brace() {
        // Regression: an empty block spanning two lines must not fold the `}` away.
        let text = "SELECT * WHERE {\n}";
        assert!(ggp_folds(text).is_empty());
    }

    #[test]
    fn subselect_group_graph_pattern() {
        let text = "SELECT * {\n  {\n    SELECT * {\n      ?s ?p ?o .\n    }\n  }\n}";
        let mut folds = ggp_folds(text);
        folds.sort();
        assert_eq!(folds, vec![(0, 5), (1, 4), (2, 3)]);
    }

    #[test]
    fn prologue_produces_imports_range() {
        let text = "PREFIX ex: <http://example.org/>\nSELECT * {\n  ?s ?p ?o .\n}";
        let (tree, _) = parse(text);
        let ranges = compute_folding_ranges(&tree, text);
        let imports: Vec<_> = ranges
            .iter()
            .filter(|range| range.kind == Some(FoldingRangeKind::Imports))
            .collect();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].start_line, 0);
    }

    #[test]
    fn query_without_group_graph_pattern_has_no_ggp_folds() {
        let text = "PREFIX ex: <http://example.org/>";
        assert!(ggp_folds(text).is_empty());
    }
}
