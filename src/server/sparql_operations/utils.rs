use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    parse_query,
};

/// Wrap a SELECT query in an outer `SELECT * WHERE { ... }` and append
/// `LIMIT`/`OFFSET` clauses, leaving any surrounding prologue/values intact.
///
/// Returns `None` if there is nothing to do (no limit and zero offset) or if
/// the input does not parse as a query containing a SELECT.
pub(crate) fn add_limit_offset_to_query(
    query: &str,
    limit: Option<usize>,
    offset: usize,
) -> Option<String> {
    if limit.is_none() && offset == 0 {
        return None;
    }
    let syntax_tree = QueryUnit::cast(parse_query(query).0)?;
    let select_query = syntax_tree.select_query()?;
    let limit_clause = limit.map_or(String::new(), |limit| format!("LIMIT {limit}\n"));
    Some(format!(
        "{}SELECT * WHERE {{\n{}\n}}\n{}OFFSET {}{}",
        &query[0..select_query.syntax().text_range().start().into()],
        select_query.text(),
        limit_clause,
        offset,
        &query[select_query.syntax().text_range().end().into()..]
    ))
}
