use ll_sparql_parser::parse_query;

use crate::server::message_handler::completion::context::CompletionLocation;

fn match_location_at_offset(input: &str, location: CompletionLocation, offset: u32) -> bool {
    let root = parse_query(input);
    CompletionLocation::from_position(&root, offset.into())
        .unwrap()
        .0
        == location
}

#[test]
fn localize_select_binding() {
    assert!(match_location_at_offset(
        //12345678
        "Select  {}",
        CompletionLocation::SelectBinding,
        7,
    ));
    assert!(!match_location_at_offset(
        //12345678901234567890
        "Select  Reduced ?a {}",
        CompletionLocation::SelectBinding,
        0,
    ));
    assert!(match_location_at_offset(
        //12345678901234567890
        "Select Reduced ?a {}",
        CompletionLocation::SelectBinding,
        6,
    ));
    assert!(match_location_at_offset(
        //12345678901234567890
        "Select Reduced ?a {}",
        CompletionLocation::SelectBinding,
        14,
    ));
    assert!(match_location_at_offset(
        //12345678901234567890
        "Select Reduced ?a {}",
        CompletionLocation::SelectBinding,
        17,
    ));
    assert!(!match_location_at_offset(
        //12345678901234567890
        "Select Reduced ?a {}",
        CompletionLocation::SelectBinding,
        19,
    ));
    assert!(!match_location_at_offset(
        //12345678901234567890
        "Select * {}",
        CompletionLocation::SelectBinding,
        8,
    ));
    assert!(!match_location_at_offset(
        //12345678901234567890123456
        "Select * { BIND (42 AS )}",
        CompletionLocation::SelectBinding,
        23,
    ));
}

#[test]
fn localize_start_1() {
    let input = "\n";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Start,
        0
    ));
}

#[test]
fn localize_start_2() {
    let input = "S\n";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Start,
        1
    ));
}

#[test]
fn localize_solution_modifier() {
    //           0123456789012
    let input = "Select * {} \n";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::SolutionModifier,
        12
    ));
}

#[test]
fn localize_subject_1() {
    //           0123456789012
    let input = "Select * {  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Subject,
        11
    ));
}

#[test]
fn localize_subject_2() {
    //           012345678901234567890123
    let input = "Select * { ?s ?p ?o .  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Subject,
        21
    ));
}

#[test]
fn localize_subject_3() {
    //           012345678901234567890123
    let input = "Select * { ?s ?p ?o .  ?s ?p ?o }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Subject,
        22
    ));
}

#[test]
fn localize_subject_4() {
    //           0123456789012
    let input = "Select * { ?  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Subject,
        12
    ));
}

#[test]
fn localize_predicate_1() {
    //           0123456789012345
    let input = "Select * { ?a }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Predicate,
        13
    ));
}

#[test]
fn localize_predicate_2() {
    //           0123456789012345678
    let input = "Select * { <iri>  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Predicate,
        17
    ));
}

#[test]
fn localize_predicate_3() {
    let input = "Select * { \"str\"  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Predicate,
        17
    ));
}

#[test]
fn localize_predicate_4() {
    //           012345678901234567890123
    let input = "Select * { ?a ?b ?c ; }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Predicate,
        21
    ));
}

#[test]
fn localize_object_1() {
    //           01234567890123456789
    let input = "Select * { ?a ?b  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Object,
        17
    ));
}

#[test]
fn localize_object_2() {
    //           01234567890123456789012
    let input = "Select * { ?a <iri>   }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Object,
        20
    ));
}

#[test]
fn localize_object_3() {
    //           01234567890123456789012
    let input = "Select * { ?a ?a ?b,  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Object,
        21
    ));
}
