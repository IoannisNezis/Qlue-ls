<h1 align="center">
  🦀 Qlue-ls 🦀
</h1>

⚡Qlue-ls (pronounced "clueless") is a *blazingly fast* [language server](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification) for [SPARQL](https://de.wikipedia.org/wiki/SPARQL),  
written in Rust 🦀, build for the web.

If you want to use Qlue-ls, check out the [documentation](https://docs.qlue-ls.com).  
To learn more about the origin story of this project, read the [blog post](https://ad-blog.cs.uni-freiburg.de/post/qlue-ls-a-sparql-language-server/).

# 🚀 Capabilities

Here is a quick overview what Qlue-ls can do.  
A more detailed list can be found in the [documentation](https://docs.qlue-ls.com/03_capabilities/).

## 📐 Formatting

Formats SPARQL queries to ensure consistent and readable syntax.
Customizable options to align with preferred query styles are also implemented.

## 🩺 Diagnostics

Diagnostics provide feadback on the query.  
Diagnostics come in severity: error, warning and info

## ✨ Completion

Completion provides suggestions how the query could continue.

For completion of subjects, predicates or objects the language server sends
completion-queries to the backend and gets the completions from the knowledge-graph.  

**These completion queries have to be configured for each knowledge-graph.**

## 🛠️ Code Actions

Code action suggest complex changes to your input.  
Often in the form of a *quickfix*, to fix a diagnostic.

## ℹ️ Hover

Get information about a token on hover.

## 🕳 Jump

Quickly jump to the next or previous important location in the query.

## ❓ operation identification

Determine if a operation is a query or update.

# ⚙️  Configuration

Qlue-ls can be configured through a `qlue-ls.toml` or `qlue-ls.yml` file.

Detailed exmplanations can be found in the [documentation](https://docs.qlue-ls.com/04_configuration/).

Here is the full default configuration
```toml
[format]
align_predicates = true
align_prefixes = true
separate_prolouge = false
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
remove_unused = true
```

# 🙏 Special Thanks

* [TJ DeVries](https://github.com/tjdevries) for the inspiration and great tutorials
* [Chris Biscardi](https://github.com/christopherbiscardi) for teaching me Rust
* [Hannah Bast](https://ad.informatik.uni-freiburg.de/staff/bast) for the guidance.
