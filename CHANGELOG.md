# Change Log

All notable changes to the "Qlue-ls" project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [1.1.17] - 2025-01-24

### Added

- new configuration option `format.line_length` to control when SELECT clauses break across multiple lines

## [1.1.16] - 2025-01-23

### Added

- new configuration option `completion.same_subject_semicolon` to control whether subject completions matching the previous subject transform the trailing dot to a semicolon
- experimental feature: compact formatting

## [1.1.14] - 2025-01-22

### Changed

- If no limit is provided for "qlueLs/executeOperation" request, the full result is returned

## [1.1.13] - 2025-01-22

### Added

- new "qlueLs/listBackends" method to list loaded SPARQL services

## [1.1.12] - 2025-01-21

### Fixed

- use provided window in query execution requests

### Changed

- renamed query template variable "qlue_ls_detail" to "qlue_ls_alias"

## [1.1.11] - 2025-01-20

### Changed

- remove "Rank" from completion item documentation
- trigger completion after object completion if object_completion_suffix is enabled

## [1.1.10] - 2025-01-19

### Added

- option to add suffix " .\n" to object completion queries
- option to have a minimum of chars before subject online completions are triggered

## [1.1.9] - 2025-01-18

### Added

- code action: add rdfs:label for variable

### Changed

- use interior mutability for parse tree cache

## [1.1.7] - 2025-01-17

### Fixed

- lexer for blank node label

## [1.1.5] - 2025-01-17

### Added

- extra information in the documentation of completion items

## [1.1.4] - 2025-01-16

### Changed

- keyword completions (FILTER, BIND, OPTIONAL, etc.) are now filtered by search term prefix

## [1.1.3] - 2025-01-16

### Added

- documentation field to CompletionItem for richer completion hints

### Changed

- refactored completion item label and detail rendering

### Fixed

- aggregate completion variable
- completion trigger token detection at end of query with trailing empty nodes

## [1.1.2] - 2025-01-13

### Fixed

- spaces in variable name completions

## [1.1.1] - 2025-01-13

### Changed

- capitalize snippet label

### Fixed

- "remove prefix declaration" quickfix

## [1.1.0] - 2025-12-27

### Added

- access token to qlueLs/executeOperation request

## [1.0.0] - 2025-12-27

### BREAKING

The request for "qlueLs/executeQuery" has been renamed to "qlueLs/executeOperation".  
The result schema of this request also changed.

### added

- construct support
- update support

### fixed

- adjust to breaking changes in the parser

## [0.24.1] - 2025-12-26

### fixed

- handle cancel notification mid result stream

## [0.24.0] - 2025-12-26

### added

- "qlueLs/cancelQuery" notification to cancel a running query

## [0.23.1] - 2025-12-17

### fixed

- lazy sparql result reader

## [0.23.0] - 2025-12-14

### Added

- more sparql engines
- lazy sparql result reader

### Fixed

- read qlever exception

## [0.21.0] - 2025-12-09

### Changed

- rename backend to service in qlueLs/addBackend

## [0.20.3] - 2025-11-26

### Added

- CRLF support

### Changed

- updated rust edition to 2024

## [0.20.2] - 2025-11-25

### Fixed

- apply changes from textDocument/didChange correctly, again

## [0.20.1] - 2025-11-24

### Fixed

- predicate and object completion for non-wasm targets. (thanks to @DeaconDesperado)
- completion replacement for neovim (thanks to @DeaconDesperado)
- apply changes from textDocument/didChange correctly

### Added

- set Query-Id for executeQuery requests
- qlueLs/getBackend request to get current default backend

## [0.19.2] - 2025-11-06

### Added

- post message if wasm-target crashes

## [0.19.1] - 2025-11-03

### Added

- forward connection error for executeQuery requests
- forward SPARQL endpoint error for executeQuery requests

### Fixed

- formatting blank-node-property-list

## [0.18.0] - 2025-10-29

### Fixed

- completion queries in demo

### Added

- capability to execute queries


## [0.17.1] - 2025-10-21

### Fixed

- object completions now use correct query templates

## [0.17.0] - 2025-10-18

### Changed

- renamed completion query templates (BREAKING)

### Added

- foldingRange for Prologue

### Fixed

- formatting emojis
- indentation after contract same subject triples

## [0.15.1] - 2025-09-30

### Changed

- core textedit apply algorithm

### Added

- more hover documentation for keywords
- aggregate completions for implicit GROUP BY

## [0.14.2] - 2025-09-23

### Added

- aggregate completions

### Fixed

- prevent trailing newline for monaco based editors

## [0.14.1]

### Fixed

- cli formatting, ignored newlines

## [0.14.0]

### Added

- Snippets for SPARQL 1.1 Update

### Changed

- cli format api: when path is omited, use stdin stdout

## [0.13.4]

### Fixed

- handle subject completion request gracefully
- fix formatting for codepoints with width 2 (emojis)
- fix subselect code action when emojis are present

### Added

- tracing capability

## [0.13.3]

### Added

- diagnostic and code-action for same subject triples

### Changed

- prefill order completions

## [0.13.2]

### Added

- add order-condition completions
- prepend variable completions to spo completions

### Changed

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

- New code-actions: add aggregates to result

## [0.12.1]

### Changed

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

### Changed

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

### Changed

- property list is not part of the global completion context

## [0.7.0]

### Changed

- replace tree-sitter with hand-written parser
  - this effects almost everything and behaviour changes are possible
- **breaking** identify operation type always returns a String
- update various dependencies

### Fixed

- syntax highlighting of comments in demo editor
- tokenize 'DELETE WHERE'
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

- updated and removed various dependencies

## [0.6.1]

### Fixed

- bug in formatter

## [0.6.0]

### Added

- configurable completion query timeout
- configurable completion query result limit
- development setup documentation
- debug log for completion queries
- semantic variable completions: hasHeight -> ?height
- async processing of long running requests (completion and ping)
- custom lsp message "jump", to jump to next relevant location

### Changed

- backends configuration in demo editor is now yaml not json
- completion details are in completion item label_details instead of detail 
  (gets always rendered in monaco, not just when hovering)


### Fixed

- langtag tokenization
- prefix-compression in service blocks
- variable completions
- textual rendering of rdf-terms
- various completion query templates


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
