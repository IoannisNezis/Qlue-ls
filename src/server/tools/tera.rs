use indoc::indoc;
use tera::Tera;

pub(super) fn init() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_templates([
        (
            "subject_completion.rq",
            indoc! {
                "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                 PREFIX dblp: <https://dblp.org/rdf/schema#>
                 SELECT ?qlue_ls_value (?alias AS ?qlue_ls_label) ?qlue_ls_detail WHERE {
                   {
                     SELECT ?qlue_ls_value ?alias ?count ?qlue_ls_detail WHERE {
                       {
                         {
                           SELECT ?qlue_ls_value (COUNT(?paper) AS ?count) WHERE {
                             ?paper dblp:publishedIn ?qlue_ls_value . 
                           }
                           GROUP BY ?qlue_ls_value
                         }
                         BIND (?qlue_ls_value AS ?alias)
                                 Values ?qlue_ls_detail { \"Journal\" }
                       }
                       UNION {
                         {
                           SELECT ?qlue_ls_value (COUNT(?paper) AS ?count) WHERE {
                             ?paper dblp:authoredBy ?qlue_ls_value
                           }
                           GROUP BY ?qlue_ls_value
                         }
                         ?qlue_ls_value rdfs:label ?alias
                                 Values ?qlue_ls_detail { \"Author\" }
                       }
                       UNION {
                         {
                           SELECT ?qlue_ls_value (COUNT(?author) AS ?count) WHERE {
                             ?qlue_ls_value dblp:authoredBy ?author
                           }
                           GROUP BY ?qlue_ls_value
                         }
                         ?qlue_ls_value dblp:title ?alias
                                 Values ?qlue_ls_detail { \"Item\" }
                       }
                     }
                    INTERNAL SORT BY ?alias}
                   FILTER REGEX(STR(?alias),\"^{{search_term}}\")
                 }
                 ORDER BY DESC(?count)
                 LIMIT 100
                 OFFSET 0"
            },
        ),
        (
            "object_completion.rq",
            indoc! {
                "PREFIX dblps: <https://dblp.org/rdf/schema-2020-07-01#>
                 PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                 PREFIX dblp: <https://dblp.org/rdf/schema#>
                 PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                 {% for prefix in prefixes %}
                 PREFIX {{prefix.0}}: <{{prefix.1}}>
                 {% endfor %}
                 SELECT ?qlue_ls_value ?qlue_ls_label ?qlue_ls_detail WHERE {
                   {
                     SELECT ?qlue_ls_value WHERE {
                       {{context}} ?qlue_ls_value
                     }
                     GROUP BY ?qlue_ls_value
                   }
                   OPTIONAL {
                     ?qlue_ls_value dblp:name ?name .
                   }
                   OPTIONAL {
                     ?qlue_ls_value rdfs:label ?label .
                   }
                   OPTIONAL {
                     ?qlue_ls_value dblp:comment ?qlue_ls_detail .
                   }
                   BIND (COALESCE(?name, ?label, ?qlue_ls_value) AS ?qlue_ls_label)
                   {% if search_term %}
                   FILTER REGEX(STR(?qlue_ls_value),\"^{{search_term}}\")
                   {% endif %}
                 }
                 ORDER BY DESC(?count)
                 LIMIT 100
                 "
            },
        ),
        (
            "predicate_completion.rq",
            indoc! {
               "{% for prefix in prefixes %}
                PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endfor %}
                PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                SELECT * WHERE {
                  {
                    SELECT ?qlue_ls_value  WHERE {
                      {{context}}
                    }
                    GROUP BY ?qlue_ls_value
                  }
                }
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
                SELECT ?qlue_ls_value WHERE {
                  {{entity}} rdfs:label ?label .
                  OPTIONAL {
                      {{entity}} rdfs:comment ?comment .
                  }
                  Bind(COALESCE(?comment, ?label) as ?qlue_ls_value)
                }
                LIMIT 1
               "
            },
        ),
    ])
    .expect("Templates should be valid");
    tera
}
