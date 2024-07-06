use indoc::indoc;
use tree_sitter::Parser;

use crate::message_handler::formatting::format_helper;

fn format_and_compare(ugly_query: &str, pretty_query: &str) {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_sparql::language())
        .unwrap();
    let tree = parser.parse(ugly_query, None).unwrap();
    let formatted_query = format_helper(
        &ugly_query.to_string(),
        &mut tree.root_node().walk(),
        0,
        "  ",
    );
    assert_eq!(formatted_query, pretty_query);
}
#[test]
fn prologue() {
    let ugly_query = indoc!(
        "
              PReFIX   namespace:  <uri>

            prefix namespace:  <uri>
            SELECT * {}"
    );
    let pretty_query = indoc!(
        "PREFIX namespace: <uri>
         PREFIX namespace: <uri>

         SELECT * {}"
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn nesting_indentation() {
    let ugly_query = "SELECT * {{{SELECT * {?a ?a ?a}}}}";
    let pretty_query = indoc!(
        "SELECT * {
           {
             {
               SELECT * {
                 ?a ?a ?a
               }
             }
           }
         }"
    );
    format_and_compare(ugly_query, pretty_query);
}
#[test]
fn alternating_group_graph_pattern() {
    let ugly_query = indoc!(
        "SELECT * {
             ?a ?c ?b .{
             } ?a ?b ?c
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
           ?a ?c ?b .
           {}
           ?a ?b ?c
         }"
    );
    format_and_compare(ugly_query, pretty_query);
}

#[test]
fn union() {
    let ugly_query = indoc!(
        "SELECT * {
           {} UNION {}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           UNION
           {}
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn select_clause() {
    let ugly_query = indoc!("SELECT ( <> as ?a) ?a  *{}");
    let pretty_query = indoc!("SELECT (<> AS ?a) ?a * {}");
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn optional() {
    let ugly_query = indoc!(
        "SELECT * {
             Optional
             {
             ?a ?c ?c}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
           OPTIONAL {
             ?a ?c ?c
           }
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn minus() {
    let ugly_query = indoc!(
        "SELECT * {
             {} minus {}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           MINUS
           {}
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn graph() {
    let ugly_query = indoc!(
        "SELECT * {
             {} Graph ?a { ?a ?b  ?c}
             }"
    );
    let pretty_query = indoc!(
        "SELECT * {
           {}
           GRAPH ?a {
             ?a ?b ?c
           }
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn filter() {
    let ugly_query = indoc!("SELECT * {filter   (1>0)}");
    let pretty_query = indoc!(
        "SELECT * {
           FILTER (1 > 0)
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn binary_expression() {
    let ugly_query = indoc!(
        "SELECT * {
            filter (1 = 3+2-2.9*10/0 && 
                    1 > 2 ||
                    1 < 2 ||
                    1 <= 2 &&
                    1 >= 9 ||
                    1 != 3 ||
                    5 in (1,2,3) &&
                    6 not in (4,5,6+3))}"
    );
    let pretty_query = indoc!(
        "SELECT * {
           FILTER (1 = 3 + 2 - 2.9 * 10 / 0 && 1 > 2 || 1 < 2 || 1 <= 2 && 1 >= 9 || 1 != 3 || 5 IN (1, 2, 3) && 6 NOT IN (4, 5, 6 + 3))
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn bind() {
    let ugly_query = indoc!("SELECT * {Bind (1 as ?var )}");
    let pretty_query = indoc!(
        "SELECT * {
           BIND ( 1 AS ?var )
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn inline_data() {
    let ugly_query = indoc!("SELECT * {values ?a { 1 2 3}}");
    let pretty_query = indoc!(
        "SELECT * {
           VALUES ?a { 1 2 3 }
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn values_clause() {
    let ugly_query = indoc!("SELECT * {}values ?a { 1 2 3}");
    let pretty_query = indoc!(
        "SELECT * {}

         VALUES ?a { 1 2 3 }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn solution_modifier() {
    let ugly_query = indoc!(
        "SELECT * WHERE {}
           GROUP by ( 2 AS ?a )
          HAVING (2 > 2) (1 > 2)
            order BY ASC (?c)
         OFfSET 3 LiMIT 3"
    );
    let pretty_query = indoc!(
        "SELECT * WHERE {}
         GROUP BY ( 2 AS ?a )
         HAVING (2 > 2) (1 > 2)
         ORDER BY ASC (?c)
         OFFSET 3 LIMIT 3"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn dataset_clause() {
    let ugly_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         SELECT  ?name
         FROM    <http://example.org/foaf/aliceFoaf>
         WHERE   { ?x foaf:name ?name }"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>

         SELECT ?name FROM <http://example.org/foaf/aliceFoaf> WHERE {
           ?x foaf:name ?name
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn construct() {
    let ugly_query = indoc!(
        "PREFIX foaf:    <http://xmlns.com/foaf/0.1/>
         PREFIX vcard:   <http://www.w3.org/2001/vcard-rdf/3.0#>
         CONSTRUCT   { <http://example.org/person#Alice> vcard:FN ?name }
         WHERE       { ?x foaf:name ?name }"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>
         PREFIX vcard: <http://www.w3.org/2001/vcard-rdf/3.0#>

         CONSTRUCT {
           <http://example.org/person#Alice> vcard:FN ?name
         }
         WHERE {
           ?x foaf:name ?name
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn describe() {
    let ugly_query = indoc!(
        "PREFIX foaf:   <http://xmlns.com/foaf/0.1/>
         DESCRIBE ?x ?y <http://example.org/>
         WHERE    {?x foaf:knows ?y}"
    );
    let pretty_query = indoc!(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>

         DESCRIBE ?x ?y <http://example.org/> WHERE {
           ?x foaf:knows ?y
         }"
    );
    format_and_compare(ugly_query, pretty_query)
}

#[test]
fn ask() {
    let ugly_query = indoc!(
        r#"PREFIX foaf:    <http://xmlns.com/foaf/0.1/>
           ASK  { ?x foaf:name  "Alice" }
           "#
    );
    let pretty_query = indoc!(
        r#"PREFIX foaf: <http://xmlns.com/foaf/0.1/>

           ASK {
             ?x foaf:name "Alice"
           }"#
    );
    format_and_compare(ugly_query, pretty_query)
}