# :gear: Configuration

Qlue-ls can be configured through a `qlue-ls.toml` or `qlue-ls.yml` file.

Here is an example configuration:

```toml
[format]
align_prefixes = false
align_predicates = true
separate_prologue = false
capitalize_keywords = true
insert_spaces = true
tab_size = 2
where_new_line = false
filter_same_line = true
line_length = 120
contract_triples = false

[completion]
timeout_ms = 5000
result_size_limit = 100
subject_completion_trigger_length = 3
object_completion_suffix = true
variable_completion_limit = 10
same_subject_semicolon = true

auto_line_break = false

[prefixes]
add_missing = true
remove_unused = false
```

## Format settings

### format.align_prefixes

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Indent all prefixes s.t. they align:

```sparql
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
```

### format.align_predicates

| Type     | Default |
| ---------| --------|
| boolean  | true   |

Indent predicates in a property list s.t. they align:

```sparql
?subject rdf:type ?type .
         rdfs:label label
```

This setting also controls on-type formatting: when pressing Enter after `;`, the new line is indented to the predicate column when `true`, or to depth + one tab when `false`.
See [On-type Formatting](02_capabilities.md#on-type-formatting).


### format.separate_prologue

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Separate Prologue from query with a line break:

```sparql
PREFIX rdf:      <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
```

### format.capitalize_keywords

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Capitalize all keywords.


### format.insert_spaces

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Use spaces and not tabs.

### format.tab_size


| Type     | Default |
| ---------| --------|
| integer  | 2       |

How wide a tab is.

### format.where_new_line

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Insert newline before each WHERE:

```sparql
SELECT *
WHERE {}
```


### format.filter_same_line

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Allow trailing filter statements:

```sparql
SELECT * WHERE {
    ?a ?b ?c Filter(?a)
}
```

### format.line_length

| Type     | Default |
| ---------| --------|
| integer  | 120     |

Maximum line length before SELECT clauses break across multiple lines.
When a SELECT clause exceeds this length, each binding is placed on a new line with proper indentation:

```sparql
SELECT ?variable1
       ?variable2
       ?variable3
WHERE {}
```

### format.contract_triples

| Type     | Default |
| ---------| --------|
| boolean  | false   |

When enabled, automatically contracts consecutive triples with the same subject using semicolon notation.

```sparql
# Before (contract_triples = false)
SELECT * WHERE {
  ?person <name> ?name .
  ?person <age> ?age .
}

# After (contract_triples = true)
SELECT * WHERE {
  ?person <name> ?name ;
          <age> ?age .
}
```

When `align_predicates = true` (default), the contracted predicates are aligned with the first predicate.
When `align_predicates = false`, a fixed 2-space indentation is used.

### format.keep_empty_lines

| Type     | Default |
| ---------| --------|
| boolean  | false   |

When enabled, preserves intentional blank lines from the original source.
Consecutive blank lines are collapsed into a single empty line.

## Completion settings

### completion.timeout_ms

| Type     | Default |
| ---------| --------|
| integer  | 5000    |

Time (in ms) a completion query is allowed to take.

### completion.result_size_limit

| Type     | Default |
| ---------| --------|
| integer  | 100     |

The result size of a completion query.

### completion.subject_completion_trigger_length

| Type     | Default |
| ---------| --------|
| integer  | 3     |

The amount of chars (actually bytes) required to trigger a subject completion.
This concerns online completions and not variable or construct completions!

### completion.object_completion_suffix

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Automatically append ` .\n` after object completions.
This helps with writing triple patterns by automatically closing the statement.

### completion.variable_completion_limit

| Type     | Default |
| ---------| --------|
| integer  | none    |

Maximum number of variable completions to suggest.
When not set (default), all variables in the query are suggested.
Set this to limit suggestions when queries have many variables.

### completion.same_subject_semicolon

| Type     | Default |
| ---------| --------|
| boolean  | true    |

When completing a subject that matches the previous triple's subject, transform the completion to use semicolon notation instead of starting a new triple.
This allows you to continue adding predicates to the same subject without repeating it.

For example, if you have:

```sparql
?person rdf:type foaf:Person .
```

And you complete `?person` as the next subject, it will transform to:

```sparql
?person rdf:type foaf:Person ;
        |
```

Where `|` represents your cursor position, ready to add another predicate.

## On-type formatting settings

### auto_line_break

| Type     | Default |
| ---------| --------|
| boolean  | false   |

When enabled, typing `;` or `.` after a valid triple automatically inserts a newline with correct indentation.

- After `;`: new line indented to predicate column (or base + tab when `align_predicates = false`)
- After `.`: new line at base brace-depth indent

This only applies when the preceding triple is syntactically complete (has subject, predicate, and object). Incomplete triples are ignored.

See [On-type Formatting](02_capabilities.md#on-type-formatting) for more details.

## Prefix settings

### prefixes.add_missing

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Define missing prefix declarations as soon as they are needed.

### prefixes.remove_unused

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Remove prefix declarations if they are not used.  

!!! warning

    Turn off if you plan to define custom prefixes!!


## Backend settings

Backends represent knowledge bases that the LSP can connect to in order to provide smart completions.

### backend.name

| Type     | Required | Default  |
| ---------| ---------|----------|
| string   | yes      | —        |

The name of the backend.

### backend.url

| Type     | Required | Default  |
| ---------| ---------|----------|
| string   | yes      | —        |

The URL for the SPARQL service to query for completions.

### backend.healthCheckUrl

| Type     | Required | Default  |
| ---------| ---------|----------|
| string   | no       | —        |

A URL to probe to test availability of the backend for completions.

### backend.engine

| Type     | Required | Default  |
| ---------| ---------|----------|
| string   | no       | —        |

The SPARQL engine type. One of `QLever`, `GraphDB`, `Virtuoso`, `MillenniumDB`, `Blazegraph`, `Jena`.

### backend.requestMethod

| Type    | Required | Default  |
| --------| ---------|----------|
| string  | no       | GET      |

The HTTP method to use for requests to this backend service.

### backend.prefixMap

| Type                  | Required | Default  |
| ----------------------| ---------|----------|
| map<string, string>   | no       | {}       |

A mapping of RDF prefixes to IRIs that will be predefined for use with this backend.

### backend.default

| Type    | Required | Default  |
| --------| ---------|----------|
| boolean | no       | false    |

Whether to activate this backend by default at server start. Inactive backends can be activated via an LSP
request to `qlueLs/updateDefaultBackend`.

### backend.queries

| Type                  | Required | Default  |
| ----------------------| ---------|----------|
| map<string, string>   | no       | {}       |

The SPARQL queries used for completions by the LSP. See the [Completion Queries](05_completion_queries.md)
guide for details.

### backend.additionalData

| Type   | Required | Default  |
| -------| ---------|----------|
| any    | no       | —        |

Arbitrary additional data to attach to the backend configuration.
