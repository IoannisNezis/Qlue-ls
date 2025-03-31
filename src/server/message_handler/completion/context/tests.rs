use ll_sparql_parser::parse_query;

use crate::server::message_handler::completion::context::CompletionLocation;

use super::{get_anchor_token, get_continuations, get_location, get_trigger_token};

fn match_location_at_offset(input: &str, matcher: CompletionLocation, offset: u32) -> bool {
    location(input, offset) == matcher
}

fn location(input: &str, offset: u32) -> CompletionLocation {
    let root = parse_query(input);
    let trigger_token = get_trigger_token(&root, offset.into());
    let anchor = trigger_token.and_then(get_anchor_token);
    let continuations = get_continuations(&root, &anchor);
    get_location(&anchor, &continuations)
}

#[test]
fn localize_select_binding() {
    assert!(matches!(
        location("Select  {}", 7),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(!matches!(
        location("Select  Reduced ?a {}", 0),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        location("Select  Reduced ?a {}", 6),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        location("Select  Reduced ?a {}", 14),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        location("Select  Reduced ?a {}", 17),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        location("Select  Reduced ?a {}", 19),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(!matches!(
        location("Select * {}", 8),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(!matches!(
        location("Select * { BIND (42 AS )}", 23),
        CompletionLocation::SelectBinding(_),
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
    assert!(matches!(
        location(input, 13),
        CompletionLocation::Predicate(_),
    ));
}

#[test]
fn localize_predicate_2() {
    //           0123456789012345678
    let input = "Select * { <iri>  }";

    assert!(matches!(
        location(input, 17),
        CompletionLocation::Predicate(_),
    ));
}

#[test]
fn localize_predicate_3() {
    let input = "Select * { \"str\"  }";
    assert!(matches!(
        location(input, 17),
        CompletionLocation::Predicate(_),
    ));
}

#[test]
fn localize_predicate_4() {
    //           012345678901234567890123
    let input = "Select * { ?a ?b ?c ; }";
    assert!(matches!(
        location(input, 21),
        CompletionLocation::Predicate(_),
    ));
}

#[test]
fn localize_object_1() {
    //           01234567890123456789
    let input = "Select * { ?a ?b  }";
    assert!(matches!(location(input, 17), CompletionLocation::Object(_)));
}

#[test]
fn localize_object_2() {
    //           01234567890123456789012
    let input = "Select * { ?a <iri>   }";
    assert!(matches!(location(input, 20), CompletionLocation::Object(_)));
}

#[test]
fn localize_object_3() {
    //           01234567890123456789012
    let input = "Select * { ?a ?a ?b,  }";
    assert!(matches!(location(input, 21), CompletionLocation::Object(_)));
}
