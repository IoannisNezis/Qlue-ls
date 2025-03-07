use crate::{ast::TriplesBlock, parse_query, SyntaxNode};

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
        triples[2].triples_block().unwrap().syntax.to_string(),
        "?s ?p ?o . ?a  ?b ?c .     ?x ?y  ?z"
    );
}
