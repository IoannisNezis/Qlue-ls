# :gear: Configuration

Qlue-ls can be configured through a `qlue-ls.toml` or `qlue-ls.yml` file.

Here is the full default configuration
```toml
[format]
align_predicates = true
align_prefixes = true
separate_prolouge = false
capitalize_keywords = true
insert_spaces = true
tab_size = 10
where_new_line = true
filter_same_line = true

[completion]
timeout_ms = 5000
result_size_limit = 42
```
