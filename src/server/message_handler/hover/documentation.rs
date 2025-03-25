use indoc::indoc;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub(super) fn get_docstring_for_kind(kind: SyntaxKind) -> Option<String> {
    match kind {
        SyntaxKind::FILTER => Some(indoc! {
            "### **FILTER**
              The `FILTER` keyword is used to restrict the results by applying a boolean condition.

             ---

             # **Example:**

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

             **Example:**

             ```sparql
             PREFIX foaf: <http://xmlns.com/foaf/0.1/>

             SELECT ?name
             WHERE {
               ?person foaf:name ?name .
             }
             ```"
        }),
        _ => None,
    }.map(|s| s.to_string())
}
