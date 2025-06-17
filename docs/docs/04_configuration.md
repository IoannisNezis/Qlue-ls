# :gear: Configuration

Qlue-ls can be configured through a `qlue-ls.toml` or `qlue-ls.yml` file.

Here is the full default configuration:

```toml
[format]
align_prefixes = false
align_predicates = true
separate_prologue = false
capitalize_keywords = true
insert_spaces = true
tab_size = 2
where_new_line = true
filter_same_line = true

[completion]
timeout_ms = 5000
result_size_limit = 100

[prefixes]
add_missing = true
remove_unused = false
```

## Formatt settings

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


### format.separate_prolouge

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Separate Prolouge from query with a line break:

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

## Prefix settings

### prefix.add_missing

| Type     | Default |
| ---------| --------|
| boolean  | true    |

Define missing prefix declarations as soon as they are needed.

### prefix.remove_unused

| Type     | Default |
| ---------| --------|
| boolean  | false   |

Remove prefix declarations if they are not used.  

!!! warning

    Turn off if you plan to define custom prefixes!!

