use indoc::indoc;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub(super) fn get_docstring_for_kind(kind: SyntaxKind) -> Option<String> {
    match kind {
        SyntaxKind::FILTER => Some(indoc! {
            "### **FILTER**
              The `FILTER` keyword is used to restrict the results by checking a condition.

             ---
           
             #### Example:

             ```sparql
             SELECT ?name WHERE {
               ?person foaf:name ?name .
               ?person foaf:age ?age .
               FILTER (?age > 20)
             }
             ```"
        }),
        SyntaxKind::PREFIX => Some(indoc! {
        "### **PREFIX**

             The `PREFIX` keyword defines a namespace prefix to simplify the use of URIs in the query.

             ---

             #### Example:

             ```sparql
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>

             SELECT ?name
             WHERE {
               ?person foaf:name ?name .
             }
             ```"
        }),
        SyntaxKind::SELECT => Some(indoc! {
            "### **SELECT**
            The `SELECT` keyword is used to specify the variables that should appear in the query results.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name ?age WHERE {
              ?person foaf:name ?name .
              ?person foaf:age ?age .
            }
            ```"
        }),
        SyntaxKind::CONSTRUCT => Some(indoc! {
            "### CONSTRUCT
            The CONSTRUCT query returns an RDF graph based on a specified template pattern, allowing you to build new triples from existing data.

            ---

            #### Example:
            ```sparql
            CONSTRUCT { ?person foaf:knows ?friend } WHERE {
              ?person foaf:knows ?friend .
            }
            ````"
        }),
        SyntaxKind::DESCRIBE => Some(indoc! {
            "### **DESCRIBE**
            The `DESCRIBE` query returns RDF data about resources, typically returning all available information for specified URIs or variables.

            ---

            #### Example:
            ```sparql
            DESCRIBE ?person WHERE {
              ?person a foaf:Person .
            }
            ```"
        }),
        SyntaxKind::ASK => Some(indoc! {
            "### **ASK**
            The `ASK` query returns a boolean indicating whether the query pattern has any matching data in the dataset.

            ---

            #### Example:
            ```sparql
            ASK WHERE {
              ?person foaf:name 'Alice' .
            }
            ```"
        }),
        SyntaxKind::WHERE => Some(indoc! {
            "### **WHERE**
            The `WHERE` keyword introduces the graph pattern that defines the conditions for matching data.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name WHERE {
              ?person foaf:name ?name .
            }
            ```"
        }),
        SyntaxKind::OPTIONAL => Some(indoc! {
            "### **OPTIONAL**
            The `OPTIONAL` keyword allows including additional patterns that may or may not match.  
            If they don’t match, the query still returns results, with missing values left unbound.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name ?email WHERE {
              ?person foaf:name ?name .
              OPTIONAL { ?person foaf:mbox ?email . }
            }
            ```"
        }),
        SyntaxKind::UNION => Some(indoc! {
            "### **UNION**
            The `UNION` keyword combines results from multiple graph patterns, returning matches from either.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name WHERE {
              { ?person foaf:name ?name . }
              UNION
              { ?org foaf:name ?name . }
            }
            ```"
        }),
        SyntaxKind::GRAPH => Some(indoc! {
            "### **GRAPH**
            The `GRAPH` keyword is used to query named graphs in a dataset.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name WHERE {
              GRAPH <http://example.org/graph1> {
                ?person foaf:name ?name .
              }
            }
            ```"
        }),
        SyntaxKind::ORDER => Some(indoc! {
            "### **ORDER BY**
            The `ORDER BY` keyword sorts the results according to given variables or expressions.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name ?age WHERE {
              ?person foaf:name ?name .
              ?person foaf:age ?age .
            }
            ORDER BY ?age
            ```"
        }),
        SyntaxKind::LIMIT => Some(indoc! {
            "### **LIMIT**
            The `LIMIT` keyword restricts the number of results returned.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name WHERE {
              ?person foaf:name ?name .
            }
            LIMIT 5
            ```"
        }),
        SyntaxKind::OFFSET => Some(indoc! {
            "### **OFFSET**
            The `OFFSET` keyword skips a given number of results, often used with `LIMIT` for pagination.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?name WHERE {
              ?person foaf:name ?name .
            }
            OFFSET 10
            LIMIT 5
            ```"
        }),

        SyntaxKind::LANG => Some(indoc! {
            "### **LANG**
            The `LANG` function returns the language tag of a literal, if any.
            
            ---
            
            #### Example:
            ```sparql
            SELECT ?label WHERE {
              ?s rdfs:label ?label .
              FILTER(LANG(?label) = 'en')
            }
            ```"
        }),
        SyntaxKind::COUNT => Some(indoc! {
            "### COUNT
            The COUNT aggregate function returns the number of items in a group.
            
            ---
            
            #### Example:
            ```sparql
            SELECT (COUNT(?person) AS ?numPeople) WHERE {
              ?person a foaf:Person .
            }```"
        }),
        SyntaxKind::SUM => Some(indoc! {
                "### **SUM**
                The `SUM` aggregate function returns the total sum of numeric values.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (SUM(?salary) AS ?totalSalary) WHERE {
                  ?person foaf:salary ?salary .
                }
                ```"
        }),
        SyntaxKind::AVG => Some(indoc! {
                "### **AVG**
                The `AVG` aggregate function returns the average value of a numeric expression.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (AVG(?age) AS ?averageAge) WHERE {
                  ?person foaf:age ?age .
                }
                ```"
        }),
        SyntaxKind::MIN => Some(indoc! {
                "### **MIN**
                The `MIN` aggregate function returns the minimum value of an expression.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (MIN(?age) AS ?youngest) WHERE {
                  ?person foaf:age ?age .
                }
                ```"
        }),
        SyntaxKind::MAX => Some(indoc! {
                "### **MAX**
                The `MAX` aggregate function returns the maximum value of an expression.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (MAX(?age) AS ?oldest) WHERE {
                  ?person foaf:age ?age .
                }
                ```"
        }),

        SyntaxKind::SAMPLE => Some(indoc! {
                "### **SAMPLE**
                The `SAMPLE` aggregate function returns a single sample value from a group.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (SAMPLE(?name) AS ?exampleName) WHERE {
                  ?person foaf:name ?name .
                }
                ```"
        }),

        SyntaxKind::GROUP_CONCAT => Some(indoc! {
                r#"### **GROUP_CONCAT**
                The `GROUP_CONCAT` aggregate function concatenates all values of a group into a single string, optionally separated by a delimiter.
                
                ---
                
                #### Example:
                ```sparql
                SELECT (GROUP_CONCAT(?name; separator=", ") AS ?allNames) WHERE {
                  ?person foaf:name ?name .
                }
                ```"#
        }),
        SyntaxKind::REDUCED => Some(indoc! {
                "### **REDUCED**
                The `REDUCED` modifier is similar to `DISTINCT` but allows the query engine to remove duplicates if it can, without guaranteeing complete elimination.
                
                ---
                
                #### Example:
                ```sparql
                SELECT REDUCED ?name WHERE {
                  ?person foaf:name ?name .
                }
                ```"
        }),
        SyntaxKind::DISTINCT => Some(indoc! {
                "### **DISTINCT**
                The `DISTINCT` modifier ensures that duplicate results are removed from the query results.
                
                ---
                
                #### Example:
                ```sparql
                SELECT DISTINCT ?name WHERE {
                  ?person foaf:name ?name .
                }
                ```"
        }),
        SyntaxKind::SERVICE => Some(indoc! {
                "### **SERVICE**
                The `SERVICE` keyword is used to delegate part of a query to a remote SPARQL endpoint.
                
                ---
                
                #### Example:
                ```sparql
                SELECT ?label WHERE {
                  SERVICE <http://dbpedia.org/sparql> {
                    ?s rdfs:label ?label .
                    FILTER(LANG(?label) = 'en')
                  }
                }
                ```"
        }),        _ => None,
    }.map(|s| s.to_string())
}
