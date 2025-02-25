use indoc::indoc;

use crate::server::message_handler::identification::determine_operation_type;

fn expect_type(operation: &str, expected_type: &str) {
    let determined_type =
        determine_operation_type(operation.into()).expect("could not determine operation type");
    assert_eq!(determined_type, expected_type);
}

#[test]
fn query_basic() {
    expect_type(indoc!("SELECT * WHERE { ?s ?p ?o }"), "Query");
    expect_type(indoc!(
        "SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");

    expect_type(indoc!(
        "SELECT * {
            ?s ?p ?o
        }"), "Query");
    expect_type(indoc!(
        "# some comment
        SELECT (count(?s) as ?count)
        WHERE
        {
            ?s ?p ?o
        }"), "Query");
}

#[test]
fn query_comments() {
    expect_type(indoc!(
        "# DELETE WHERE { ?s ?p ?o }
        SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");
    expect_type(indoc!(
        "# INSERT DATA { <a> <b> <c> }
        SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");
    expect_type(indoc!(
        "# many
        # comments
        # before
        # the
        # operation
        SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");
}

#[test]
fn query_prefix() {
    expect_type(indoc!(
        "PREFIX f: <foo>
        SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");
    expect_type(indoc!(
        "PREFIX : <foo>
        SELECT * WHERE {
            ?s ?p ?o
        }"), "Query");
}

#[test]
fn query_complex() {
    expect_type(indoc!(
        "# Buildings in REGION more than KM away from a transit stop
         #!TEMPLATE REGION=osmrel:1686344
         #!TEMPLATE KM=1
         PREFIX osmkey: <https://www.openstreetmap.org/wiki/Key:>
         PREFIX osmrel: <https://www.openstreetmap.org/relation/>
         PREFIX ogc: <http://www.opengis.net/rdf#>
         PREFIX geo: <http://www.opengis.net/ont/geosparql#>
         PREFIX qlss: <https://qlever.cs.uni-freiburg.de/spatialSearch/>
         SELECT ?building ?stop ?dist ?building_geometry WHERE {
           { SELECT ?building ?stop ?dist WHERE {
             %REGION% ogc:sfContains ?building . ?building osmkey:building [] . ?building geo:hasCentroid/geo:asWKT ?building_location .
         	SERVICE qlss: {
         	  _:config qlss:left ?building_location ; qlss:right ?stop_location ; qlss:numNearestNeighbors 1 ; qlss:payload ?stop ; qlss:bindDistance ?dist .
         	  { { ?stop osmkey:highway 'bus' } UNION { ?stop osmkey:public_transport 'station' } UNION { ?stop osmkey:public_transport 'platform' }
               ?stop geo:hasCentroid/geo:asWKT ?stop_location }
         	}
             FILTER (?dist > %KM%)
           } }
           ?building geo:hasGeometry/geo:asWKT ?building_geometry .
           ?stop geo:hasGeometry/geo:asWKT ?stop_geometry .
         } ORDER BY DESC(?dist)"
    ), "Query");
    // This is syntax is a QLever specific extension of SPARQL.
    expect_type(indoc!(
        "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT * WHERE {
          ?a @en@rdfs:label ?b
        }"), "Query");
}

#[test]
fn update_basic() {
    expect_type(indoc!(
        "INSERT DATA {}"), "Update");
    expect_type(indoc!(
        "DELETE DATA {
            <a> <b> <c>
        }"), "Update");
    expect_type(indoc!(
        "DELETE WHERE {
            ?s <foo> <bar>
        }"), "Update");
    expect_type(indoc!(
        "DELETE {
            ?s <foo> <bar>
        }
        INSERT {
            ?s <baz> <bar>
        }
        WHERE {
            ?s <foo> <bar>
        }"), "Update");
}

#[test]
fn update_complex() {
    expect_type(indoc!(
        "# This Update changes all foo to bar
        # But not if it is a leap day
        #!TEMPLATE REGION=osmrel:1686344
        PREFIX foo: <http://www.opengis.net/rdf#>
        PREFIX
        bar:
        <http://www.opengis.net/ont/geosparql#>
        PREFIX : <http://www.opengis.net/rdf#>
        BASE
        <http://www.opengis.net/ont/geosparql#>
        INSERT
        DATA
        {
            <a> <b> <c>
        }"
    ), "Update");
}

#[test]
fn unknown() {
    expect_type("", "Unknown");
    expect_type("THIS IS NOT SPARQL", "Unknown");
    expect_type(indoc!(
        "PREFIX foo: <bar>
        QLEVER * { ?s ?p foo:bar }"), "Unknown");
}
