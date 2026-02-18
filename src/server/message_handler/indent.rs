use ll_sparql_parser::{
    SyntaxToken,
    ast::{AstNode, Triple},
    syntax_kind::SyntaxKind,
};
use unicode_width::UnicodeWidthChar;

/// Returns the column position (in display cells) of a byte offset within `text`,
/// measured from the start of the current line.
///
/// Walks backwards from `offset` until a newline (or the start of the text),
/// accumulating Unicode display widths.
pub(crate) fn column_at_offset(text: &str, offset: usize) -> usize {
    text[..offset]
        .chars()
        .rev()
        .take_while(|c| c != &'\n')
        .fold(0, |acc, c| acc + c.width().unwrap_or(0))
}

/// Given a syntax token inside a `TriplesSameSubjectPath`, returns the display-column
/// of the first predicate (start of the `PropertyListPath`). Used for aligning
/// subsequent predicates under the first when `align_predicates` is true.
/// Returns `None` if the token is not inside a `TriplesSameSubjectPath`.
pub(crate) fn predicate_alignment_column(token: &SyntaxToken, text: &str) -> Option<usize> {
    let triple_node = token
        .parent_ancestors()
        .find(|n| n.kind() == SyntaxKind::TriplesSameSubjectPath)?;
    let triple = Triple::cast(triple_node)?;
    let offset: usize = triple
        .properties_list_path()?
        .syntax()
        .text_range()
        .start()
        .into();
    Some(column_at_offset(text, offset))
}

/// Returns the brace nesting depth of a syntax token, counting all brace-delimited
/// constructs in the SPARQL grammar:
/// `GroupGraphPattern`, `ConstructTemplate`, `QuadData`, `QuadPattern`,
/// `QuadsNotTriples`, `InlineDataOneVar`, `InlineDataFull`, and the short-form
/// `CONSTRUCT WHERE { TriplesTemplate }`.
pub(crate) fn brace_nesting_depth(token: &SyntaxToken) -> usize {
    let mut depth = 0;
    let mut prev_kind: Option<SyntaxKind> = None;

    for node in token.parent_ancestors() {
        match node.kind() {
            SyntaxKind::GroupGraphPattern
            | SyntaxKind::ConstructTemplate
            | SyntaxKind::QuadData
            | SyntaxKind::QuadPattern
            | SyntaxKind::QuadsNotTriples
            | SyntaxKind::InlineDataOneVar
            | SyntaxKind::InlineDataFull => depth += 1,

            // NOTE: ConstructQuery's short form 'WHERE' '{' TriplesTemplate? '}' embeds
            // braces directly without a named wrapper node. Detected by the child on the
            // path being TriplesTemplate (vs. ConstructTemplate or WhereClause).
            SyntaxKind::ConstructQuery => {
                if prev_kind == Some(SyntaxKind::TriplesTemplate) {
                    depth += 1;
                }
            }

            _ => {}
        }
        prev_kind = Some(node.kind());
    }

    depth
}

#[cfg(test)]
mod tests {
    use ll_sparql_parser::{SyntaxNode, parse_query, parse_update};
    use text_size::TextSize;

    use super::brace_nesting_depth;

    fn token_at(tree: SyntaxNode, offset: usize) -> ll_sparql_parser::SyntaxToken {
        tree.token_at_offset(TextSize::from(offset as u32))
            .right_biased()
            .expect("no token at offset")
    }

    #[test]
    fn where_clause_depth_1() {
        //                           0         1         2
        //                           012345678901234567890123456
        let (tree, _) = parse_query("SELECT * WHERE { ?s ?p ?o }");
        // ?s is at offset 17, inside one GroupGraphPattern
        let token = token_at(tree, 17);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn optional_depth_2() {
        //                           0         1         2         3         4
        //                           01234567890123456789012345678901234567890
        let (tree, _) = parse_query("SELECT * WHERE { OPTIONAL { ?s ?p ?o } }");
        // ?s is at offset 28, inside two GroupGraphPatterns (WHERE + OPTIONAL)
        let token = token_at(tree, 28);
        assert_eq!(brace_nesting_depth(&token), 2);
    }

    #[test]
    fn construct_template_returns_0_not_1() {
        //                           0         1         2         3         4
        //                           01234567890123456789012345678901234567890
        let (tree, _) = parse_query("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }");
        // ?s at offset 12 is inside ConstructTemplate — depth should be 1
        let token = token_at(tree, 12);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn construct_where_shortform_returns_0_not_1() {
        //                           0         1         2
        //                           0123456789012345678901234567
        let (tree, _) = parse_query("CONSTRUCT WHERE { ?s ?p ?o }");
        // ?s at offset 18 is inside the inline { } of ConstructQuery — depth should be 1
        let token = token_at(tree, 18);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quad_data_insert_returns_0_not_1() {
        //                            0         1         2
        //                            012345678901234567890123456
        let (tree, _) = parse_update("INSERT DATA { <s> <p> <o> }");
        // <s> at offset 14 is inside QuadData — depth should be 1
        let token = token_at(tree, 14);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quad_data_delete_returns_0_not_1() {
        //                            0         1         2
        //                            012345678901234567890123456
        let (tree, _) = parse_update("DELETE DATA { <s> <p> <o> }");
        // <s> at offset 14 is inside QuadData — depth should be 1
        let token = token_at(tree, 14);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quad_pattern_delete_where_returns_0_not_1() {
        //                            0         1         2
        //                            01234567890123456789012345678
        let (tree, _) = parse_update("DELETE WHERE { <s> <p> <o> }");
        // <s> at offset 15 is inside QuadPattern (DeleteWhere) — depth should be 1
        let token = token_at(tree, 15);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quad_pattern_delete_clause_returns_0_not_1() {
        //                            0         1         2         3
        //                            01234567890123456789012345678901234567
        let (tree, _) = parse_update("DELETE { ?s ?p ?o } WHERE { ?s ?p ?o }");
        // ?s at offset 9 is inside QuadPattern (DELETE clause) — depth should be 1
        let token = token_at(tree, 9);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quad_pattern_insert_clause_returns_0_not_1() {
        //                            0         1         2         3
        //                            0123456789012345678901234567890123456789
        let (tree, _) = parse_update("INSERT { ?s ?p ?o } WHERE { ?s ?p ?o }");
        // ?s at offset 9 is inside QuadPattern (INSERT clause) — depth should be 1
        let token = token_at(tree, 9);
        assert_eq!(brace_nesting_depth(&token), 1);
    }

    #[test]
    fn quads_not_triples_returns_0_not_2() {
        //                            0         1         2         3         4
        //                            01234567890123456789012345678901234567890
        let (tree, _) = parse_update("INSERT DATA { GRAPH <g> { <s> <p> <o> } }");
        // <s> at offset 26 is inside QuadsNotTriples (inside QuadData) — depth should be 2
        let token = token_at(tree, 26);
        assert_eq!(brace_nesting_depth(&token), 2);
    }

    #[test]
    fn inline_data_one_var_returns_1_not_2() {
        //                           0         1         2         3
        //                           012345678901234567890123456789012345
        let (tree, _) = parse_query("SELECT * WHERE { VALUES ?x { <a> } }");
        // <a> at offset 29 is inside InlineDataOneVar (inside GroupGraphPattern) — depth should be 2
        let token = token_at(tree, 29);
        assert_eq!(brace_nesting_depth(&token), 2);
    }

    #[test]
    fn inline_data_full_returns_1_not_2() {
        //                           0         1         2         3         4
        //                           01234567890123456789012345678901234567890
        let (tree, _) = parse_query("SELECT * WHERE { VALUES (?x) { (<a>) } }");
        // <a> at offset 32 is inside InlineDataFull (inside GroupGraphPattern) — depth should be 2
        let token = token_at(tree, 32);
        assert_eq!(brace_nesting_depth(&token), 2);
    }
}
