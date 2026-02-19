# Formatting Test Cases

This directory contains file-based formatting tests. Each subdirectory represents a single test case.

## Directory Structure

```
cases/
  basic/
    input.sparql      # Required: unformatted SPARQL query
    expected.sparql   # Required: expected formatted output
  align_prefixes/
    input.sparql
    expected.sparql
    settings.toml     # Optional: custom FormatSettings
  ...
```

## Adding a New Test Case

1. Create a new directory with a descriptive name (use snake_case)
2. Add `input.sparql` with the unformatted query
3. Add `expected.sparql` with the expected output
4. Optionally add `settings.toml` with custom format settings

The test runner automatically discovers all test cases - no code changes required.

## Settings Format

The `settings.toml` file uses camelCase field names matching `FormatSettings`:

```toml
# Example settings.toml
alignPrefixes = true
alignPredicates = false
separatePrologue = true
capitalizeKeywords = false
whereNewLine = true
filterSameLine = false
tabSize = 4
insertSpaces = true
lineLength = 80
contractTriples = false
compact = 60
```

All settings are optional - defaults are used for any unspecified fields.

## Available Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `alignPredicates` | bool | true | Align predicates in property lists |
| `alignPrefixes` | bool | false | Align PREFIX declarations |
| `separatePrologue` | bool | false | Add blank line after prologue |
| `capitalizeKeywords` | bool | true | Uppercase SPARQL keywords |
| `insertSpaces` | bool | true | Use spaces instead of tabs |
| `tabSize` | u8 | 2 | Indentation size |
| `whereNewLine` | bool | false | Put WHERE on new line |
| `filterSameLine` | bool | true | Keep FILTER on same line as triple |
| `compact` | u32 | (none) | Compact formatting threshold |
| `lineLength` | u32 | 120 | Line length for SELECT wrapping |
| `contractTriples` | bool | false | Contract triples with same subject |
| `keepEmptyLines` | bool | false | Preserve intentional blank lines (consecutive lines collapsed) |

## Running Tests

```bash
# Run all file-based formatting tests
cargo test --test formatting_file_based

# Run specific test case(s) using FILTER env var
FILTER=basic cargo test --test formatting_file_based

# Run with verbose output
cargo test --test formatting_file_based -- --nocapture
```

The `FILTER` environment variable filters test cases by name (substring match).
For example, `FILTER=basic` runs all cases containing "basic" in their directory name.
