use rowan::TextRange;

use crate::{ast::TriplesBlock, parse_query, print_full_tree, SyntaxNode};

fn walk(node: SyntaxNode, mut path: Vec<usize>) -> Option<SyntaxNode> {
    if path.is_empty() {
        return Some(node);
    }
    let head = path.remove(0);
    if let Some(child) = node.children().nth(head) {
        return walk(child, path);
    }
    return None;
}

#[test]
fn triples_block() {
    let input = "SELECT * { ?s ?p ?o . ?a ?b ?c .     ?x ?y ?z}";
    let root = parse_query(input);
    let node = walk(root, vec![0, 0, 1, 0, 0, 0]).unwrap();
    let triples_block = TriplesBlock::cast(node).unwrap();
    triples_block.triples();
    println!("{:?}", triples_block.triples());
    println!("{:?}", triples_block.triples()[0].subject());
}
