use ll_sparql_parser::{SyntaxToken, parse_query, syntax_kind::SyntaxKind};

use crate::server::message_handler::completion::environment::CompletionLocation;

use super::{get_anchor_token, get_continuations, get_location, get_trigger_token};

fn match_location_at_offset(input: &str, matcher: CompletionLocation, offset: u32) -> bool {
    location(input, offset) == matcher
}

fn location(input: &str, offset: u32) -> CompletionLocation {
    let (root, _) = parse_query(input);
    let trigger_token = get_trigger_token(&root, offset.into());
    let anchor = trigger_token.and_then(|token| get_anchor_token(token, offset.into()));
    let continuations = get_continuations(&root, &anchor);
    get_location(&anchor, &continuations, offset.into())
}

#[test]
fn find_anchor_path() {
    let (root, _) = parse_query("Select * {?s ^}");
    let trigger_token = get_trigger_token(&root, 14.into());
    let anchor = trigger_token
        .and_then(|token| get_anchor_token(token, 14.into()))
        .unwrap();
    assert_eq!(anchor.kind(), SyntaxKind::Zirkumflex)
}

#[test]
fn localize_select_binding() {
    assert!(matches!(
        //        0123456789
        location("Select  {}", 7),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(!matches!(
        //        012345678901234567890
        location("Select  Reduced ?a {}", 0),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        //        012345678901234567890
        location("Select  Reduced ?a {}", 14),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        //        012345678901234567890
        location("Select  Reduced ?a {}", 17),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        //        012345678901234567890
        //        0123456678901233456789
        location("Select  Reduced ?a {}", 16),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        //        012345678901234567890
        location("Select * {}", 8),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(matches!(
        //        012345678901234567890
        location("Select (3 as ?x) {}", 16),
        CompletionLocation::SelectBinding(_),
    ));

    assert!(!matches!(
        //        012345678901234567890
        location("Select * { BIND (42 AS )}", 23),
        CompletionLocation::SelectBinding(_),
    ));
}

#[test]
fn localize_blank_property() {
    //           012345678901234567890123
    let input = "Select * { ?s ?p [] }";
    assert!(matches!(
        location(input, 18),
        CompletionLocation::BlankNodeProperty(_),
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
fn localize_subject_5() {
    //           0123456789012345
    let input = "Select * { ?var  }";
    assert!(match_location_at_offset(
        input,
        CompletionLocation::Subject,
        15
    ));
}

#[test]
fn localize_predicate_1() {
    //           0123456789012345
    let input = "Select * { ?a }";
    assert!(matches!(
        location(input, 14),
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
fn localize_predicate_5() {
    //           012345678901234567890123
    let input = "Select * { ?a a }";
    assert!(matches!(
        location(input, 15),
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

#[test]
fn localize_object_4() {
    //           012345678901234567890123456
    let input = "Select * { ?a rdfs:label ?  }";
    assert!(matches!(location(input, 26), CompletionLocation::Object(_)));
}

fn trigger_token_at(input: &str, offset: u32) -> Option<SyntaxToken> {
    let (root, _) = parse_query(input);
    get_trigger_token(&root, offset.into())
}

#[test]
fn trigger_token_at_end() {
    //           01234567890123456789012
    let input = "Select * { ?a ?a ?b }";

    assert!(matches!(
        trigger_token_at(input, 21).unwrap().kind(),
        SyntaxKind::RCurly
    ));
}

#[test]
fn localize_a() {
    //           01234567890123456789012
    let input = "Select * { ?a a  }";
    assert!(matches!(location(input, 16), CompletionLocation::Object(_),));
}

#[test]
fn search_term_includes_all_error_tokens() {
    use super::get_search_term;
    //           0123456789012345
    let input = "Select * { Ex }";
    let (root, _) = parse_query(input);

    // When typing "Ex", the parser creates two separate Error tokens: "E" and "x"
    // The search term should include both, regardless of cursor position

    // Position 12: cursor is between "E" and "x"
    let trigger_token_12 = get_trigger_token(&root, 12.into());
    let anchor_12 = trigger_token_12.and_then(|t| get_anchor_token(t, 12.into()));
    let search_term_12 = get_search_term(&root, &anchor_12, 12.into());
    assert_eq!(
        search_term_12,
        Some("Ex".to_string()),
        "At position 12 (between 'E' and 'x'), search_term should include both Error tokens"
    );

    // Position 13: cursor is after "Ex" (after the "x")
    // This is the typical position when completion is triggered after typing
    let trigger_token_13 = get_trigger_token(&root, 13.into());
    let anchor_13 = trigger_token_13.and_then(|t| get_anchor_token(t, 13.into()));
    let search_term_13 = get_search_term(&root, &anchor_13, 13.into());
    assert_eq!(
        search_term_13,
        Some("Ex".to_string()),
        "At position 13 (after 'Ex'), search_term should include both Error tokens"
    );
}

#[test]
fn localize_order_condition() {
    //           0123456789012345678901234567
    let input = "SELECT * WHERE {} ORDER BY ";
    let (root, _) = parse_query(input);
    assert!(get_trigger_token(&root, 27.into()).is_some());
}

// --- InlineData (VALUES) location tests ---

#[test]
fn localize_inline_data_one_var() {
    //           0         1         2
    //           0123456789012345678901234
    let input = "SELECT * { VALUES ?x {  } }";
    assert!(matches!(
        location(input, 23),
        CompletionLocation::InlineData((_, 0)),
    ));
}

#[test]
fn localize_inline_data_one_var_with_existing_value() {
    //           0         1         2         3
    //           0123456789012345678901234567890123
    let input = "SELECT * { VALUES ?x { <a>  } }";
    assert!(matches!(
        location(input, 27),
        CompletionLocation::InlineData((_, 0)),
    ));
}

#[test]
fn localize_inline_data_full_first_slot() {
    //           0         1         2         3
    //           01234567890123456789012345678901234
    let input = "SELECT * { VALUES (?x ?y) { (  ) } }";
    assert!(matches!(
        location(input, 30),
        CompletionLocation::InlineData((_, 0)),
    ));
}

#[test]
fn localize_inline_data_full_second_slot() {
    //           0         1         2         3         4
    //           01234567890123456789012345678901234567890123
    let input = "SELECT * { VALUES (?x ?y) { (<a>  ) } }";
    assert!(matches!(
        location(input, 33),
        CompletionLocation::InlineData((_, 1)),
    ));
}

#[test]
fn localize_inline_data_full_third_slot() {
    //           0         1         2         3         4         5
    //           012345678901234567890123456789012345678901234567890123
    let input = "SELECT * { VALUES (?x ?y ?z) { (<a> <b>  ) } }";
    assert!(matches!(
        location(input, 40),
        CompletionLocation::InlineData((_, 2)),
    ));
}

#[test]
fn localize_inline_data_full_second_row_resets_index() {
    //           0         1         2         3         4         5
    //           0123456789012345678901234567890123456789012345678901234567
    let input = "SELECT * { VALUES (?x ?y) { (<a> <b>) (  ) } }";
    // NOTE: cursor in second row, first slot — index should reset to 0
    assert!(matches!(
        location(input, 40),
        CompletionLocation::InlineData((_, 0)),
    ));
}

#[test]
fn localize_inline_data_full_second_row_second_slot() {
    //           0         1         2         3         4         5         6
    //           0123456789012345678901234567890123456789012345678901234567890123
    let input = "SELECT * { VALUES (?x ?y) { (<a> <b>) (<c>  ) } }";
    // NOTE: cursor in second row, second slot
    assert!(matches!(
        location(input, 44),
        CompletionLocation::InlineData((_, 1)),
    ));
}

#[test]
fn localize_inline_data_empty_multi_var_block_is_unknown() {
    //           0         1         2         3
    //           01234567890123456789012345678901
    let input = "SELECT * { VALUES (?x ?y) {  } }";
    // NOTE: cursor in empty block without a parenthesized row — no DataBlockValue continuation
    assert!(matches!(location(input, 28), CompletionLocation::Unknown));
}

#[test]
fn localize_inline_data_post_query_is_unknown() {
    //           0         1         2         3
    //           01234567890123456789012345
    let input = "SELECT * {} VALUES ?x {  }";
    // NOTE: post-query VALUES clause (ValuesClause) is not handled as InlineData
    assert!(matches!(location(input, 24), CompletionLocation::Unknown));
}

#[test]
fn localize_inline_data_index_beyond_declared_vars() {
    //           0         1         2         3         4         5
    //           012345678901234567890123456789012345678901234567890123
    let input = "SELECT * { VALUES (?x ?y) { (<a> <b> |) } }";
    // NOTE: cursor after two DataBlockValues — index 2 exceeds the two declared variables
    assert!(matches!(
        location(input, 38),
        CompletionLocation::InlineData((_, 2)),
    ));
}
