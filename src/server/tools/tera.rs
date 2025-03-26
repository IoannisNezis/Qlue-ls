use indoc::indoc;
use tera::Tera;

pub(super) fn init() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_templates([
        (
            "object_completion.rq",
            indoc! {
               "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                PREFIX dblp: <https://dblp.org/rdf/schema#>
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                {% for prefix in prefixes %}
                    PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endfor %}
                SELECT ?entity ?name ?count  WHERE {
                  {
                    SELECT ?entity (COUNT(?entity) AS ?count) WHERE {
                        {{context}} ?entity
                    }
                    GROUP BY ?entity
                  }
                  OPTIONAL {
                    ?entity dblp:creatorName ?creatorname .
                  }
                  OPTIONAL {
                    ?entity rdfs:label ?label .
                  }
                  BIND (COALESCE(?creatorname, ?label, ?entity) AS ?name)
                }
                ORDER BY DESC(?count)
                LIMIT 100"
            },
        ),
        (
            "predicate_completion.rq",
            indoc! {
               "{% for prefix in prefixes %}
                PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endfor %}
                SELECT ?pred (COUNT(?pred) as ?count)  WHERE {
                    {{context}} ?pred ?o .
                }
                GROUP BY ?pred
                ORDER BY DESC(?count)
                LIMIT 100
               "
            },
        ),
        (
            "hover_iri.rq",
            indoc! {
               "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                {% if prefix %}
                PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endif %}
                SELECT ?hover WHERE {
                  {{entity}} rdfs:label ?label .
                  OPTIONAL {
                      {{entity}} rdfs:comment ?comment .
                  }
                  Bind(COALESCE(?comment, ?label) as ?hover)
                }
                LIMIT 1
               "
            },
        ),
    ])
    .expect("Templates should be valid");
    tera
}
