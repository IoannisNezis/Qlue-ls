use rowan::TextRange;

use crate::{
    ast::{AstNode, GroupGraphPattern, QueryUnit, Triple, TriplesBlock, WhereClause},
    parse_query, SyntaxNode,
};

fn walk(node: SyntaxNode, mut path: Vec<usize>) -> Option<SyntaxNode> {
    if path.is_empty() {
        return Some(node);
    }
    let head = path.remove(0);
    if let Some(child) = node.children().nth(head) {
        return walk(child, path);
    }
    None
}

#[test]
fn prologue() {
    let input = "PREFIX a: <dings>\n Prefix b: <foo> SELECT ?a WHERE { ?s ?p ?o}";

    let root = parse_query(input);
    let query_unit = QueryUnit::cast(root).unwrap();
    let prologue = query_unit.prologue().unwrap();
    assert_eq!(prologue.prefix_declarations()[0].prefix().unwrap(), "a");
    assert_eq!(
        prologue.prefix_declarations()[0].uri_prefix().unwrap(),
        "<dings>"
    );

    assert_eq!(prologue.prefix_declarations()[1].prefix().unwrap(), "b");
    assert_eq!(
        prologue.prefix_declarations()[1].uri_prefix().unwrap(),
        "<foo>"
    );

    let query_unit2 = QueryUnit::cast(parse_query("SELECT * {}")).unwrap();
    assert_eq!(query_unit2.prologue(), None);
}

#[test]
fn where_clause() {
    let input = "SELECT ?a WHERE { ?s ?p ?o}";

    let root = parse_query(input);
    let node = walk(root, vec![0, 0, 1]).unwrap();
    let where_clause = WhereClause::cast(node).unwrap();
    assert_eq!(
        where_clause.where_token().unwrap().text_range(),
        TextRange::new(10.into(), 15.into())
    );
    assert_eq!(
        where_clause
            .group_graph_pattern()
            .unwrap()
            .syntax
            .to_string(),
        "{ ?s ?p ?o}"
    );
}

#[test]
fn group_graph_pattern() {
    let input = "SELECT * { ?s ?p ?o . {} ?a  ?b ?c .     ?x ?y  ?z}";
    let root = parse_query(input);
    let node = walk(root, vec![0, 0, 1, 0]).unwrap();
    let ggp = GroupGraphPattern::cast(node).unwrap();
    let triples_blocks = ggp.triple_blocks();
    assert_eq!(triples_blocks.len(), 2);
    assert_eq!(
        triples_blocks[1]
            .group_graph_pattern()
            .unwrap()
            .syntax
            .to_string(),
        "{ ?s ?p ?o . {} ?a  ?b ?c .     ?x ?y  ?z}"
    );
    assert_eq!(
        ggp.l_paren_token().unwrap().text_range(),
        TextRange::new(9.into(), 10.into())
    );
    assert_eq!(
        ggp.r_paren_token().unwrap().text_range(),
        TextRange::new(50.into(), 51.into())
    );
}

#[test]
fn triples_block() {
    let input = "SELECT * { ?s ?p ?o . ?a  ?b ?c .     ?x ?y  ?z}";
    let root = parse_query(input);
    let node = walk(root, vec![0, 0, 1, 0, 0, 0]).unwrap();
    let triples_block = TriplesBlock::cast(node).unwrap();
    let triples = triples_block.triples();
    assert_eq!(triples.len(), 3);
    assert_eq!(triples_block.triples()[0].syntax.to_string(), "?s ?p ?o");
    assert_eq!(triples_block.triples()[1].syntax.to_string(), "?a  ?b ?c");
    assert_eq!(triples_block.triples()[2].syntax.to_string(), "?x ?y  ?z");
    assert_eq!(
        triples[2].triples_block().unwrap().syntax().to_string(),
        "?s ?p ?o . ?a  ?b ?c .     ?x ?y  ?z"
    );
}

#[test]
fn ast_triple() {
    let input = "SELECT * { ?s ?p ?o ; ?p2 ?o2 ; ?p3 ?o3}";
    let root = parse_query(input);
    let node = walk(root, vec![0, 0, 1, 0, 0, 0, 0]).unwrap();
    let triple = Triple::cast(node).unwrap();
    println!(
        "{:?}",
        triple
            .variables()
            .iter()
            .map(|var| var.syntax().text().to_string())
            .collect::<Vec<String>>()
    );
}
