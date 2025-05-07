# Change Log

All notable changes to the "Qlue-ls" project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
