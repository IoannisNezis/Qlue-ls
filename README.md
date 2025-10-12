<h1 align="center">
  🦀 Qlue-ls 🦀
</h1>

<div align="center">
    <a href="https://crates.io/crates/qlue-ls">
        <img alt="crates.io" src="https://img.shields.io/crates/v/qlue-ls.svg" />
    </a>
    <a href="https://www.npmjs.com/package/qlue-ls">
        <img alt="npm" src="https://img.shields.io/npm/v/qlue-ls" />
    </a>
    <a href="https://pypi.org/project/qlue-ls">
        <img alt="PyPI" src="https://img.shields.io/pypi/v/qlue-ls" />
    </a>
</div>

⚡Qlue-ls (pronounced "clueless") is a *blazingly fast* [language server](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification)
for [SPARQL](https://de.wikipedia.org/wiki/SPARQL), written in Rust 🦀, build for the web.

💻 [Live Demo](https://qlue-ls.com)
📚 [Documentation](https://docs.qlue-ls.com)  
📝 [Project Blog Post](https://ad-blog.cs.uni-freiburg.de/post/qlue-ls-a-sparql-language-server/)  
🎓 [Thesis](https://ad-publications.cs.uni-freiburg.de/theses/Bachelor_Ioannis_Nezis_2025.pdf)  

# 🚀 Capabilities

Qlue-ls offers a wide range of LSP features tailored to SPARQL development.  
For a complete overview, see the [capabilities section](https://docs.qlue-ls.com/03_capabilities/).

## ✨ Completion

- Suggests valid continuations while typing SPARQL queries
- Backend-powered suggestions for subjects, predicates, and objects
- **Note:** Completion queries must be configured for each knowledge graph

<div align="left">
   <p>https://github.com/user-attachments/assets/207c8265-27b9-4dde-a18c-d82f7c5db4c9</p>
</div>

## 📐 Formatting

- Auto-formats SPARQL queries for consistency and readability
- Fully customizable to match your preferred coding style

<div align="left">
   <p>https://github.com/user-attachments/assets/9d80ae33-8ff0-4bdd-8a9d-fb95a632673e</p>
</div>

## 🛠️ Code Actions

- Provides smart quick-fixes for diagnostics
- Offers suggested improvements and automated edits

<div align="left">
   <p>https://github.com/user-attachments/assets/53fe75b6-71d2-4fe9-91c8-82ebda420712</p>
</div>

## ℹ️ Hover

- View contextual information by hovering over tokens

<div align="left">
   <p>https://github.com/user-attachments/assets/425e6912-c9f0-49ca-9937-6cd536ab9bc4</p>
</div>

## 🩺 Diagnostics

- Real-time feedback with severity levels: error, warning, and info
- Helps catch syntax issues and common mistakes

## 🕳 Jump

- Navigate quickly between key locations in a query

## ❓ operation identification

- Detects whether a SPARQL operation is a `query` or an `update`

# ⚙️  Configuration

Qlue-ls is configured via a qlue-ls.toml or qlue-ls.yml file.  
Full configuration options are explained in the [documentation](https://docs.qlue-ls.com/04_configuration/).

## Default Configuration

```toml
[format]
align_predicates = true
align_prefixes = true
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

# 🙏 Acknowledgements

* [TJ DeVries](https://github.com/tjdevries) - for the inspiration and fantastic tutorials
* [Hannah Bast](https://ad.informatik.uni-freiburg.de/staff/bast) - for mentorship and guidance.
