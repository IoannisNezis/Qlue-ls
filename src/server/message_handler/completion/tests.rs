use ll_sparql_parser::parse_query;

use crate::server::message_handler::completion::context::CompletionLocation;

fn match_location_at_offset(input: &str, location: CompletionLocation, offset: u32) {
    let root = parse_query(input);
    assert_eq!(
        CompletionLocation::from_position(root, offset.into()).unwrap(),
        location
    );
}

#[test]
fn localize_start_1() {
    let input = "\n";
    match_location_at_offset(input, CompletionLocation::Start, 0);
}

#[test]
fn localize_start_2() {
    let input = "S\n";
    match_location_at_offset(input, CompletionLocation::Start, 1);
}

#[test]
fn localize_end() {
    //           0123456789012
    let input = "Select * {} \n";
    match_location_at_offset(input, CompletionLocation::End, 12);
}

#[test]
fn localize_triple_or_not_1() {
    //           0123456789012
    let input = "Select * {  }";
    match_location_at_offset(input, CompletionLocation::TripleOrNotTriple, 11);
}

#[test]
fn localize_triple_or_not_2() {
    //           012345678901234567890123
    let input = "Select * { ?s ?p ?o .  }";
    match_location_at_offset(input, CompletionLocation::TripleOrNotTriple, 22);
}

#[test]
fn localize_triple_or_not_3() {
    //           012345678901234567890123
    let input = "Select * { ?s ?p ?o .  ?s ?p ?o }";
    match_location_at_offset(input, CompletionLocation::TripleOrNotTriple, 22);
}

#[test]
fn localize_triple_or_not_4() {
    //           0123456789012
    let input = "Select * { ?  }";
    match_location_at_offset(input, CompletionLocation::TripleOrNotTriple, 12);
}

#[test]
fn localize_predicate_1() {
    //           0123456789012345
    let input = "Select * { ?a  }";
    match_location_at_offset(input, CompletionLocation::Predicate, 14);
}

#[test]
fn localize_predicate_2() {
    //           0123456789012345678
    let input = "Select * { <iri>  }";
    match_location_at_offset(input, CompletionLocation::Predicate, 17);
}

#[test]
fn localize_predicate_3() {
    let input = "Select * { \"str\"  }";
    match_location_at_offset(input, CompletionLocation::Predicate, 17);
}

#[test]
fn localize_object_1() {
    //           01234567890123456789
    let input = "Select * { ?a ?b  }";
    match_location_at_offset(input, CompletionLocation::Object, 17);
}

#[test]
fn localize_object_2() {
    //           01234567890123456789012
    let input = "Select * { ?a <iri>   }";
    match_location_at_offset(input, CompletionLocation::Object, 20);
}
