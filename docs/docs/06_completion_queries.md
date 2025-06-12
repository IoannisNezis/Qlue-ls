# Completion Queries

Completion queries are send to the backend to retrieve the completion suggestions.
These Queries are individual for each knowledgebase.
The user has to define a query template for each type of online-completion.

## Completion query anatomy

Each completion query **MUST** result must contain the following variables.

| Variable          | Content                             | Example               |
| ----------------- | ----------------------------------- | --------------------- |
| `?qlue_ls_entity` | RDF tem, value to be completed      | \<book_1\>            |
| `?qlue_ls_label`  | representation of completion item   | book title            |
| `?qlue_ls_detail` | description of the completion item  | Book from author ...  |

## Templating engine

The templates are rendered by [tera](https://keats.github.io/tera/docs), a templating engine.
It provides:

- [Control structures](https://keats.github.io/tera/docs/#control-structures), like **for** and **if**
- [Data manipulation](https://keats.github.io/tera/docs/#manipulating-data), like **filters**, **tests** and **functions**

## Template Context

Each template has the following variables available:

| Variable      | Content                              | Example |
| ------------- | ------------------------------------ | ------- |
| prefixes      | list of prefix, iri pairs            | [("rdfs", "http://www.w3.org/2000/01/rdf-schema#"), ("rdf","http://www.w3.org/1999/02/22-rdf-syntax-ns#")] |
| subject       | subject of current triple            | "?sub" |
| local_context | Query pattern for the current triple | "?sub ?qlue_ls_entity []"  |
| context       | Query pattern for the constrainig part of the query | "?sub rdfs:type <Thing> . ?sub <n> 42"  |


### Custom Tests

Tests can be used against an expression to check some condition.
There are many [build in tests](https://keats.github.io/tera/docs/#built-in-tests) but also some custom SPARQL specific ones:

**variable**

Takes a string and checks if its a SPARQL variable or not.

**Example**:

```tera
{% if subject is variable %}
    Subject is a variable
{% elif %}
    Subject is not a variable
{% endif %}
```

