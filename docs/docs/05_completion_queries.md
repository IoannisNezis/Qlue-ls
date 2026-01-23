# Completion Queries

Completion queries are sent to the backend to retrieve the completion suggestions.
These queries are individual for each knowledgebase.
The user has to define a query template for each type of online-completion.

## Completion Query Anatomy

Each completion query result **MUST** contain the following variables:

| Variable          | Content                             | Example               |
| ----------------- | ----------------------------------- | --------------------- |
| `?qlue_ls_entity` | RDF term, value to be completed     | \<book_1\>            |
| `?qlue_ls_label`  | representation of completion item   | book title            |
| `?qlue_ls_alias` | description of the completion item  | Book from author ...  |

Optionally, you can include `?qlue_ls_count` to provide a relevance score (e.g., occurrence count) for sorting results.

## Query Types

There are five completion query types that can be configured:

| Query Type                              | Purpose                                         |
| --------------------------------------- | ----------------------------------------------- |
| `subjectCompletion`                     | Find subject entities                           |
| `predicateCompletionContextSensitive`   | Find predicates using surrounding query context |
| `predicateCompletionContextInsensitive` | Find predicates without using context           |
| `objectCompletionContextSensitive`      | Find objects using surrounding query context    |
| `objectCompletionContextInsensitive`    | Find objects without using context              |

Additionally, `hover` queries can be configured to fetch entity information for tooltips. These are not completion queries and have different result variable requirements (see [Hover Query](#hover-query)).

### Context-Sensitive vs Context-Insensitive

- **Context-sensitive** queries use surrounding triple patterns to narrow results. For example, if you're completing an object where the predicate is `rdf:type`, the query can use that constraint to return only classes.
- **Context-insensitive** queries provide broader fallback results when context isn't available or useful. These are simpler queries that match based on the search term alone.

## Template Context

Each template has the following variables available:

| Variable                   | Type   | Description                                                   | Example                                                                                                       |
| -------------------------- | ------ | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `prefixes`                 | list   | PREFIX declarations from the document and configuration       | `[("rdfs", "http://www.w3.org/2000/01/rdf-schema#"), ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]`  |
| `subject`                  | string | Subject of the current triple                                 | `"?sub"` or `"<http://example.org/entity>"`                                                                   |
| `local_context`            | string | Triple pattern for the completion location                    | `"?sub ?qlue_ls_entity []"`                                                                                   |
| `context`                  | string | Constraining triples from the query                           | `"?sub rdf:type <Thing> . ?sub <n> 42"`                                                                       |
| `search_term`              | string | Partial text the user is typing                               | `"boo"` (when typing "book")                                                                                  |
| `search_term_uncompressed` | string | Expanded IRI if user typed a prefixed name                    | `"http://www.wikidata.org/prop/direct/P"` (when typing `wdt:P`)                                               |
| `limit`                    | int    | Maximum results (from settings)                               | `50`                                                                                                          |
| `offset`                   | int    | Pagination offset                                             | `0`                                                                                                           |
| `entity`                   | string | The entity being hovered (hover queries only)                 | `"<http://example.org/entity>"`                                                                               |

## Templating Engine

The templates are rendered by [Tera](https://keats.github.io/tera/docs), a templating engine.
It provides:

- [Control structures](https://keats.github.io/tera/docs/#control-structures), like **for** and **if**
- [Data manipulation](https://keats.github.io/tera/docs/#manipulating-data), like **filters**, **tests** and **functions**

## Custom Tests

Tests can be used against an expression to check some condition.
There are many [built-in tests](https://keats.github.io/tera/docs/#built-in-tests) but also some custom SPARQL-specific ones:

### variable

Takes a string and checks if it's a SPARQL variable.

**Example**:

```tera
{% if subject is variable %}
    Subject is a variable
{% else %}
    Subject is not a variable
{% endif %}
```

### containing

Takes a string and checks if it contains a given substring.

**Example**:

```tera
{% if context is containing("rdf:type") %}
    Context includes a type constraint
{% endif %}
```

## Example Queries

Below are simplified, generic examples for each query type. These can be adapted to your specific knowledgebase.

### Subject Completion

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      ?qlue_ls_entity ?p ?o .
      {% if search_term %}
      ?qlue_ls_entity rdfs:label ?label .
      FILTER(STRSTARTS(LCASE(?label), LCASE("{{ search_term }}")))
      {% endif %}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
}
```

### Predicate Completion (Context-Sensitive)

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      {{ context }} {{ local_context }}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
}
```

### Predicate Completion (Context-Insensitive)

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      {{ local_context }}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
}
```

### Object Completion (Context-Sensitive)

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      {{ context }} {{ local_context }}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
  {% if search_term %}
  FILTER(CONTAINS(LCASE(STR(?qlue_ls_label)), LCASE("{{ search_term }}")))
  {% endif %}
}
```

### Object Completion (Context-Insensitive)

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      {{ local_context }}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
  {% if search_term %}
  FILTER(CONTAINS(LCASE(STR(?qlue_ls_label)), LCASE("{{ search_term }}")))
  {% endif %}
}
```

**Note**: The `{{ context }}` variable renders to an empty string when not available, so explicit `{% if context %}` checks are only needed when you want fundamentally different query structures based on context presence.

### Hover Query

Hover queries fetch information about an entity for display in tooltips. Unlike completion queries, hover queries use the `entity` template variable and have different result variable requirements:

| Variable          | Content                            |
| ----------------- | ---------------------------------- |
| `?qlue_ls_label`  | Label/name of the entity           |
| `?qlue_ls_alias` | Description or additional details  |

```sparql
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?qlue_ls_label ?qlue_ls_alias WHERE {
  {{ entity }} rdfs:label ?qlue_ls_label .
  OPTIONAL { {{ entity }} rdfs:comment ?qlue_ls_alias }
}
LIMIT 1
```

## Tips and Tricks

### Prefix Declarations

Always include the prefix loop to inherit prefixes from the document and configuration:

```tera
{% for prefix in prefixes %}
PREFIX {{prefix.0}}: <{{prefix.1}}>
{% endfor %}
```

### Search Term Filtering

- Use `search_term` for label/text matching
- Use `search_term_uncompressed` when the user types a prefixed IRI (e.g., `wdt:P31`)
- Check both for robust filtering:

```tera
{% if search_term_uncompressed %}
FILTER(STRSTARTS(STR(?qlue_ls_entity), "{{ search_term_uncompressed }}"))
{% elif search_term %}
FILTER(CONTAINS(LCASE(?qlue_ls_label), LCASE("{{ search_term }}")))
{% endif %}
```

### Adapting to Subject Type

Use the `variable` test to adapt queries based on whether the subject is bound:

```tera
{% if subject is variable %}
  {# Subject is a variable like ?s, use broader matching #}
{% else %}
  {# Subject is a specific IRI, narrow results to that entity #}
{% endif %}
```

### Sub-Select Queries for Performance

Sub-select queries are a powerful technique to speed up completion queries. By performing aggregation and limiting in an inner query, you reduce the number of entities that need label/detail lookups:

```sparql
SELECT ?qlue_ls_entity ?qlue_ls_label ?qlue_ls_alias ?qlue_ls_count WHERE {
  {
    # Inner query: find and rank entities efficiently
    SELECT ?qlue_ls_entity (COUNT(*) AS ?qlue_ls_count) WHERE {
      {{ context }} {{ local_context }}
    }
    GROUP BY ?qlue_ls_entity
    ORDER BY DESC(?qlue_ls_count)
    LIMIT {{ limit }}
  }
  # Outer query: fetch labels only for the top results
  OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
  OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
}
```

This pattern is especially effective when:

- The knowledgebase has many entities but you only need a few results
- Label lookups are expensive (e.g., federated queries or large literal indexes)
- You want to order by aggregated values like counts

### Graceful Fallbacks

Use OPTIONAL for non-critical fields:

```sparql
OPTIONAL { ?qlue_ls_entity rdfs:label ?qlue_ls_label }
OPTIONAL { ?qlue_ls_entity rdfs:comment ?qlue_ls_alias }
```

### Performance Tips

- Always use `LIMIT {{ limit }}` to cap results
- Put selective filters early in the query
- Use sub-select queries to limit results before expensive operations like label lookups
