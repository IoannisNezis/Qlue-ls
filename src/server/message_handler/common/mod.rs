//! Shared analysis helpers used by multiple LSP feature handlers.

use std::collections::HashMap;

use ll_sparql_parser::{
    SyntaxNode,
    ast::{AstNode, SelectQuery, Var},
    syntax_kind::SyntaxKind,
};

/// Find all occurrences of the variable `trigger` that denote the same variable.
///
/// Two occurrences of a variable with the same name denote the same variable,
/// unless they are separated by a scope boundary. Scope boundaries are:
///
/// - **SubSelect**: occurrences inside and outside a sub-select are connected
///   iff the sub-select projects the variable (explicitly, via `AS`, or via `SELECT *`).
/// - **Union branch**: occurrences in different branches of a `UNION` are separate,
///   unless the variable also occurs outside the union in the enclosing scope,
///   in which case all of them are connected through that occurrence.
///
/// Connectivity is transitive: the result is the connected component of `trigger`.
pub(crate) fn find_variable_occurrences(trigger: &Var) -> Vec<Var> {
    let name = trigger.var_name();
    let root = trigger
        .syntax()
        .ancestors()
        .last()
        .expect("ancestors always yields self");

    // NOTE: all occurrences of the variable, in document order
    let occurrences: Vec<Var> = root
        .descendants()
        .filter_map(Var::cast)
        .filter(|var| var.var_name() == name)
        .collect();

    // NOTE: region 0 is the root region (the top-level query scope)
    let mut regions: Vec<SyntaxNode> = vec![root.clone()];
    let mut region_ids: HashMap<SyntaxNode, usize> = HashMap::from([(root, 0)]);

    let occurrence_regions: Vec<usize> = occurrences
        .iter()
        .map(|var| register_regions(var.syntax(), &mut regions, &mut region_ids))
        .collect();

    let mut components = UnionFind::new(regions.len());

    // NOTE: a sub-select is connected to its enclosing region iff it projects the variable
    for (id, node) in regions.iter().enumerate() {
        if let Some(sub_select) = SelectQuery::cast(node.clone())
            && sub_select
                .visible_variables()
                .iter()
                .any(|var| var.var_name() == name)
        {
            components.union(id, parent_region_id(node, &region_ids));
        }
    }

    let trigger_component = occurrences
        .iter()
        .position(|var| var.syntax() == trigger.syntax())
        .map(|index| components.find(occurrence_regions[index]))
        .expect("trigger variable should be an occurrence of its own name");

    occurrences
        .into_iter()
        .zip(occurrence_regions)
        .filter(|&(_, region)| components.find(region) == trigger_component)
        .map(|(var, _)| var)
        .collect()
}

/// Check if `name` is a valid SPARQL variable name (without the leading `?` or `$`).
///
/// This follows the `VARNAME` production of the SPARQL grammar:
///
/// ```text
/// VARNAME ::= ( PN_CHARS_U | [0-9] ) ( PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040] )*
/// ```
pub(crate) fn is_valid_variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(is_var_name_start_char) && chars.all(is_var_name_char)
}

/// `PN_CHARS_U | [0-9]`
fn is_var_name_start_char(c: char) -> bool {
    matches!(c,
        'A'..='Z'
        | 'a'..='z'
        | '_'
        | '0'..='9'
        | '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}'
        | '\u{00F8}'..='\u{02FF}'
        | '\u{0370}'..='\u{037D}'
        | '\u{037F}'..='\u{1FFF}'
        | '\u{200C}'..='\u{200D}'
        | '\u{2070}'..='\u{218F}'
        | '\u{2C00}'..='\u{2FEF}'
        | '\u{3001}'..='\u{D7FF}'
        | '\u{F900}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFFD}'
        | '\u{10000}'..='\u{EFFFF}')
}

/// `PN_CHARS_U | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]`
fn is_var_name_char(c: char) -> bool {
    is_var_name_start_char(c)
        || matches!(c, '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
}

/// Register all scope boundaries enclosing `node` and return the region id
/// of the innermost one (the region `node` belongs to).
fn register_regions(
    node: &SyntaxNode,
    regions: &mut Vec<SyntaxNode>,
    region_ids: &mut HashMap<SyntaxNode, usize>,
) -> usize {
    let mut innermost: Option<usize> = None;
    for boundary in node
        .ancestors()
        .filter(|token| token.kind() == SyntaxKind::SubSelect)
    {
        let id = *region_ids.entry(boundary.clone()).or_insert_with(|| {
            regions.push(boundary);
            regions.len() - 1
        });
        innermost.get_or_insert(id);
    }
    // NOTE: no enclosing boundary -> root region
    innermost.unwrap_or(0)
}

/// Region id of the scope enclosing the boundary node `boundary`.
fn parent_region_id(boundary: &SyntaxNode, region_ids: &HashMap<SyntaxNode, usize>) -> usize {
    boundary
        .parent()
        .into_iter()
        .flat_map(|parent| parent.ancestors())
        .find(|node| node.kind() == SyntaxKind::SubSelect)
        .and_then(|node| region_ids.get(&node).copied())
        .unwrap_or(0)
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, mut node: usize) -> usize {
        while self.parent[node] != node {
            self.parent[node] = self.parent[self.parent[node]];
            node = self.parent[node];
        }
        node
    }

    fn union(&mut self, u: usize, v: usize) {
        let (root_u, root_v) = (self.find(u), self.find(v));
        if root_u != root_v {
            self.parent[root_u] = root_v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ll_sparql_parser::parse_query;

    /// Parse `query`, take the `trigger`-th occurrence (0-based, in document
    /// order) of the variable `name` and assert that
    /// [`find_variable_occurrences`] returns exactly the occurrences with the
    /// given indices (0-based, in document order).
    fn check(query: &str, name: &str, trigger: usize, expected: &[usize]) {
        let (root, _) = parse_query(query);
        let vars: Vec<Var> = root
            .descendants()
            .filter_map(Var::cast)
            .filter(|var| var.var_name() == name)
            .collect();
        assert!(
            trigger < vars.len(),
            "test setup: trigger index {} out of range ({} occurrences of ?{})",
            trigger,
            vars.len(),
            name
        );
        let mut indices: Vec<usize> = find_variable_occurrences(&vars[trigger])
            .iter()
            .map(|occurrence| {
                vars.iter()
                    .position(|var| var.syntax() == occurrence.syntax())
                    .expect("every occurrence should be one of the collected vars")
            })
            .collect();
        indices.sort();
        assert_eq!(indices, expected);
    }

    #[test]
    fn same_scope() {
        check(
            "SELECT ?x WHERE { ?x <p> ?y . ?x <q> ?z }",
            "x",
            0,
            &[0, 1, 2],
        );
    }

    #[test]
    fn all_syntactic_positions_in_one_scope() {
        // NOTE: select clause, triple, filter, bind, values and order by
        check(
            "SELECT ?x WHERE {
                ?x <p> ?y .
                FILTER(?x > 3)
                BIND((?x + 1) AS ?z)
                VALUES ?x { 1 2 }
            } ORDER BY ?x",
            "x",
            2,
            &[0, 1, 2, 3, 4, 5],
        );
    }

    #[test]
    fn other_variable_names_are_not_occurrences() {
        check(
            "SELECT * WHERE { ?x <p> ?xy . ?xy <q> ?x }",
            "x",
            0,
            &[0, 1],
        );
    }

    #[test]
    fn sub_select_projected_variable_connects() {
        let query = "SELECT ?x WHERE {
            ?x <p> ?y .
            { SELECT ?x WHERE { ?x <q> ?z } }
        }";
        check(query, "x", 0, &[0, 1, 2, 3]);
        check(query, "x", 3, &[0, 1, 2, 3]);
    }

    #[test]
    fn sub_select_hidden_variable_is_separate() {
        // NOTE: the inner ?x is not projected, so it is a different variable
        let query = "SELECT ?x WHERE {
            ?x <p> ?y .
            { SELECT ?y WHERE { ?x <q> ?z } }
        }";
        check(query, "x", 0, &[0, 1]);
        check(query, "x", 2, &[2]);
    }

    #[test]
    fn sub_select_star_projects_everything() {
        check(
            "SELECT * WHERE {
                ?x <p> ?y .
                { SELECT * WHERE { ?x <q> ?z } }
            }",
            "x",
            0,
            &[0, 1],
        );
    }

    #[test]
    fn sub_select_star_does_not_project_hidden_variables() {
        // NOTE: ?x inside the innermost sub-select is hidden by `SELECT ?y`,
        // so the outer `SELECT *` does not make it visible either
        let query = "SELECT * WHERE {
            ?x <p> ?y .
            { SELECT * WHERE { { SELECT ?y WHERE { ?x <q> ?z } } } }
        }";
        check(query, "x", 0, &[0]);
        check(query, "x", 1, &[1]);
    }

    #[test]
    fn sub_select_alias_target_connects_outward() {
        // NOTE: `(?a AS ?x)` introduces ?x for the outside
        check(
            "SELECT ?x WHERE {
                { SELECT (?a AS ?x) WHERE { ?a <p> ?b } }
                ?x <q> ?c .
            }",
            "x",
            0,
            &[0, 1, 2],
        );
    }

    #[test]
    fn sub_select_alias_source_stays_inside() {
        // NOTE: the ?a in `(?a AS ?x)` belongs to the sub-select scope
        let query = "SELECT ?x WHERE {
            { SELECT (?a AS ?x) WHERE { ?a <p> ?b } }
            ?x <q> ?a .
        }";
        check(query, "a", 0, &[0, 1]);
        check(query, "a", 2, &[2]);
    }

    #[test]
    fn valid_variable_names() {
        for name in [
            "x", "xyz", "X", "x1", "1x", "42", "_x", "_", "übung", "変数", "x·y", "x‿y", "😀",
        ] {
            assert!(is_valid_variable_name(name), "?{name} should be valid");
        }
    }

    #[test]
    fn invalid_variable_names() {
        for name in [
            "", " ", "x y", "x-y", "x.y", "?x", "$x", "·x", "x\n", "x?", "x$", "(x)", "x,y",
        ] {
            assert!(!is_valid_variable_name(name), "?{name} should be invalid");
        }
    }
}
