# Change Log

All notable changes to the "Qlue-ls" project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.15.1] - 2025-09-30

### changed

- core textedit apply algorithm

## added

- more hover documentation for keywords
- aggregate completions for implicit GROUP BY

## [0.14.2] - 2025-09-23

### added

- aggregate completions

### fixed

- prevent trailing newline for monaco based editors

## [0.14.1]

### fixed

- cli formatting, ignored newlines

## [0.14.0]

### added

- Snippets for SPARQL 1.1 Update

### changed

- cli format api: when path is omited, use stdin stdout

## [0.13.4]

### fixed

- handle subject completion request gracefull
- fix formatting for codepoints with with 2 (emojis)
- fix subselect code action when emojis are present

### added

- tracing capability

## [0.13.3]

### added

- diagnostic and code-action for same subject triples

### changed

- prefill order completions

## [0.13.2]

### Added

- add order-condition completions
- prepend variable completions to spo completions

### changed

- add to result code-action: insert before aggregates
- filter & filter-lang code-action: insert after Dot

## [0.13.1]

### Added

- add new code-action "transform into subselect"

## [0.13.0]

### Added

- object variable replacements

## [0.12.3]

### Fixed

- localize blank-node-property in anon

## [0.12.2]

### Added

- New code-acitons: add aggreagtes to result

## [0.12.1]

### changed

- set default settings 'remove_unused' to false

### Added

- vim mode for demo
- Lang-Filter code action for objects

### Fixed

- prefix expansion filter

## [0.12.0]

### Fixed

- some typos: also in settings

## [0.11.0]

### Added

- custom capability: get default settings
- custom capability: change settings

## [0.10.0]

### Added

- custom capability: determine what type of operation is present

### changed

- when typing a prefix and a ":", completion now works

## [0.9.1]

### Fixed

- deduplicate automatic prefix declaration

## [0.9.0]

### Added

- automatically declare and undeclare prefixes

### Fixed

- completion localization after "a"

## [0.8.0]

### Added

- jump to previous important position

### Fixed

- when jumping to the end of the top ggp and its not empty the formatting is now fixed

## [0.7.2]

### Added

- diagnostic: when group by is used: are the selected variables in the group by clause?
- diagnostic: when a variable is assigned in the select clause, was it already defined?

### Fixed

- property list completion context

## [0.7.1]

## Changed

- property list is not part of the global completion context

## [0.7.0]

### Changed

- replace tree-sitter with hand-written parser
  - this effects almost everything and behaviour changes are possible
- **breaking** identify operation type always returns a String
- update various dependencies

### Fixed

- syntax highlighting of comments in demo editor
- tokeize 'DELETE WHERE'
- tokenize comments

## [0.6.4]

### Added

- context sensitivity for completions

### Changed

- Jump location after solution modifiers

### Fixed

- localization for inverse path completions

## [0.6.3]

### Added

- online completion support for bin target

## [0.6.2]

### Changed

- updated and removed variouse dependencies

## [0.6.1]

### Fixed

- bug in formatter

## [0.6.0]

### Added

- configurable completion query timeout
- configurable completion query result limit
- development setup documentation
- debug log for completion queries
- samantic variable completions: hasHeight -> ?height
- async processing of long running requests (completion and ping)
- custom lsp message "jump", to jump to next relevant location

### Changed

- backends configuration in demo editor is now yaml not json
- completion details are in completion item label_details instead of detail 
  (gets always rendered in monaco, not just when hovering)


### Fixed

- langtag tokenization
- prefix-compression in service blocks
- varable completions
- textual rendering of rdf-terms
- variouse completion query templates


## [0.5.6] - 2025-04-01

### Fixed

- formatting comments with correct indentation


## [0.5.3] - 2025-03-15

### Fixed

- formatting construct where queries

## [0.5.2] - 2025-03-15

### Added

- sub select snippet
- code action: filter variable
- quickfix for "unused-prefix"

### Fixed

- add to result for vars in sub select binding

## [0.5.1] - 2025-03-15

### Fixed

- tokenize PNAME_LN

### Added

- code action: add variable to result

## [0.5.0] - 2025-03-15

### Added

- ll parser
- cursor localization for completion
- completions for select bindings
- completions for solution modifiers

## [0.4.0] - 2025-02-25

### Added

- function to determine type (Query or Update)

## [0.3.5] - 2025-02-16

### Fixed

- formatting distinct keyword in aggregate
- formatting modify
- formatting describe

## [0.3.4] - 2025-02-03

### Added

- formatting support for any utf-8 input

## [0.3.3] - 2025-02-02

### Fixed

- Fixed bugs in formatter

## [0.3.2] - 2025-01-31

### Added

- stability test for formatter

### Fixed

- fixed typo in diagnostic
- reimplemented formatting options for new formatting algorithm

## [0.3.1] - 2025-01-30

### Added

- formatting inline format statements

### Fixed

- formatting input with comments at any location

## [0.3.0] - 2025-01-20

### Added

- new format option "check": dont write anything, just check if it would

## [0.2.4] - 2025-01-20

### Fixed

- add trailing newline when formatting with format cli subcommand

## [0.2.3] - 2025-01-12

### Fixed

- positions are (by default) utf-16 based, i changed the implementation to respect this

## [0.2.2] - 2025-01-09

### Fixed

- handle textdocuments-edits with utf-8 characters

## [0.2.1] - 2025-01-09

### Fixed

- formatting strings with commas

## [0.2.0] - 2025-01-09

### Added

- new code-action: declare prefix
- example for monaco-editor with a language-client attached to this language-server
- formatter subcommand uses user-configuration
- this CHANGELOG

### Fixed

- format subcommand writeback-bug
- formatting of Blank and ANON nodes

### Changed

- format cli subcommand: --writeback option, prints to stdout by default
