# :rocket: Capabilities

Qlue-s provides SPARQL support to your editor (or tool).
These are structured in Capabilities.

## ✨ Completion

Completion provides suggestions how the query could continue.

Completions are invoked automatically by your editor or by the user
(usually by the key combination `ctrl` + `space`). They are also triggered when the user types `?`.


Qlue-ls provides different types of completions:

### Variable completion

When a user types a word beginning with `?`, all variables in scope will be returned.

<figure class="video_container">
  <video controls="true" allowfullscreen="true">
    <source src="../assets/completion_var.mp4" type="video/mp4">
  </video>
</figure>

!!! warning

    Variable completion should work everywhere where variables are allowed.
    However, note that collecting variables after the cursor is harder then before the cursor
    and might not work as expected.

### SPO completion

!!! warning

    For SPO completions a backend has to be configured

**S**ubject, **P**redicate, **O**bject completions are triggered when the cursor is
in a [GraphPattern](https://www.w3.org/TR/sparql11-query/#rGroupGraphPattern).

Qlue-ls sends 2 queries to the backend, retrieving possible continuations.
One with constraining context, one without.
If the context-sensitive query failed, the context-free one is used as a fallback.


Note that the quality of the result depends on the query, while the speed depends on the
triple store.

To get really good SPO completions, [custom completion queries](05_completion_queries.md) are required.

## 📐 Formatting

Format SPARQL queries to ensure consistent and readable syntax.
Customizable options to align with preferred query styles are also implemented.


## ⌨️ On-type Formatting

Qlue-ls supports on-type formatting with three trigger characters: `\n` (Enter), `;`, and `.`.

### Enter key

When the trigger character `\n` (Enter) is pressed, Qlue-ls adjusts the indentation of the new line automatically.

The most useful case is after a **semicolon** in a triple pattern.
After typing `;` and pressing Enter, the cursor lands at the column of the first predicate, ready to continue the property list:

```sparql
SELECT * WHERE {
  ?person rdf:type foaf:Person ;
          rdfs:label ?label .
#  ^ cursor lands here after Enter
}
```

The indentation strategy is controlled by `format.align_predicates`:

| `align_predicates` | New line is indented to…                             |
|:-------------------|:-----------------------------------------------------|
| `true` (default)   | Column of the first predicate in the current triple  |
| `false`            | Brace-depth indent + one tab unit                    |

Outside of a `;` continuation, pressing Enter always produces the plain brace-depth indent.

### Auto line break (`;` and `.` triggers)

When the `auto_line_break` setting is enabled, typing `;` or `.` after a **valid** triple automatically inserts a newline with correct indentation:

- **Semicolon (`;`)**: Inserts a newline indented to the predicate column (when `align_predicates = true`) or base indent + one tab (when `align_predicates = false`).
- **Dot (`.`)**: Inserts a newline at the base brace-depth indent, ready for a new triple.

```sparql
# With auto_line_break = true, typing ";" after "?o" produces:
?s ?p ?o;
   |  # cursor here, ready for next predicate

# Typing "." after "?o" produces:
?s ?p ?o.
|  # cursor here, ready for new triple
```

This feature only activates when the triple is syntactically valid (has subject, predicate, and object). Invalid or incomplete triples are ignored.

See [Configuration](03_configuration.md#auto_line_break) for details.

## 🩺 Diagnostics

Diagnostics provide feadback on the query.
Diagnostics come in severity: ❌ error, ⚠️ warning and ℹ️ info.

Here is a complete list of diagnostics qlue-ls can provide:

| Type        | Name                         | Description                                       |
|:------------|:-----------------------------|:--------------------------------------------------|
| ❌ error    | syntax error                 | the query contains a syntax error                 |
| ❌ error    | undeclared prefix            | a used prefix is not declared                     |
| ❌ error    | ungrouped select variable    | a selected variable is not in the group by clause |
| ❌ error    | groupby star selection       | `*` is selected in a query with a group by clause |
| ❌ error    | invalid projection variable  | projection variable is already defined            |
| ⚠️  warning | unused prefix                | a declared prefix is not used                     |
| ⚠️  warning | duplicate prefix declaration | the same prefix is declared multiple times        |
| ℹ️  info    | uncompacted uri              | a raw uncompacted uri is used                     |
| ℹ️  info    | same subject                 | multiple triples have the same subject            |

## ℹ️ Hover

For example when the user hovers a `FILTER`  the server returns a explanation about what a Filter is

and how to use it.

When a backend is configured, the server will access to knowledge-graph to get information about the token.
For example if the user hovers `osmrel:62768` and a hover request is send, Qlue-ls will respond with
**"Freiburg im Breisgau"** as this is the label of `osmrel:62768`.

!!! note

    The query used to retrieve information about a iri, can be configured.

## 🛠️ Code Actions

Code action suggest complex changes to your input.
Often in the form of a *quickfix*, to fix a diagnostic.

| name                      | description                                    | diagnostic                             |
|:--------------------------|:-----------------------------------------------|:---------------------------------------|
| shorten uri               | shorten uri into compacted form                | uncompacted uri                        |
| declare prefix            | declares undeclared prefix (if known)          | undeclared prefix                      |
| remove prefix declaration | remove an unused or duplicate prefix           | unused prefix / duplicate prefix       |
| contract triples          | contract triples with same subject             | same subject                           |
| shorten all uri's         | shorten all uri's into compacted form          |                                        |
| add to result             | add variable to selected result                |                                        |
| add aggregate to result   | add aggregate (COUNT, MIN, …) over a variable  |                                        |
| filter variable           | add filter for this variable                   |                                        |
| add label                 | add rdfs:label with a language filter          |                                        |
| lang-filter               | add language filter for object variable        |                                        |
| transform into subselect  | make a select into a subselect                 |                                        |

## ✏️ Rename

Rename a variable and all occurrences that denote the same variable.

Scope boundaries are respected:
sub-selects connect only through projected variables, and `UNION` branches
are treated as separate scopes unless bridged by an outer occurrence.

The new name is validated against the SPARQL
[VARNAME](https://www.w3.org/TR/sparql11-query/#rVARNAME) grammar,
a leading `?` or `$` is tolerated.

## 🔍 Find References

List all occurrences of the variable under the cursor.

The result follows the same scope rules as [rename](#rename):
only occurrences that denote the same variable are returned.

## 🖍️ Document Highlight

When the cursor rests on a variable, all occurrences of that variable
in the query are highlighted.

Like [find references](#find-references), this respects scope boundaries,
but the highlights only cover the current document.

