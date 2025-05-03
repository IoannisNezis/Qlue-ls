<h1 align="center">
  🦀 Qlue-ls 🦀
</h1>

⚡Qlue-ls (pronounced "clueless") is a *blazingly fast* [language server](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification) for [SPARQL](https://de.wikipedia.org/wiki/SPARQL), written in Rust 🦀.

> [!CAUTION]
> This Project is still in an early stage.  
> Only the format capability is production ready.  
> The rest is experimental.

# 🚀 Getting Started

## 📦 Installation

Qlue-ls is available on [crate.io](https://crates.io/crates/qlue-ls):

```shell
cargo install --bin qlue-ls qlue-ls
```

And on [PyPI](https://pypi.org/project/qlue-ls/):

```shell
pipx install qlue-ls
```

You can also build it from source:

```shell
git clone https://github.com/IoannisNezis/Qlue-ls.git
cd Qlue-ls
cargo build --release --bin qlue-ls
```

## CLI Usage

To run Qlue-ls as **formatter** run:

```shell
qlue-ls format <PATH>
```

To run Qlue-ls as **language server** run:

```shell
qlue-ls server
```

This will create a language server listening on stdio.

## with Neovim

After you installed the language server, add this to your `init.lua`:

```lua
vim.api.nvim_create_autocmd({ 'FileType' }, {
  desc = 'Connect to Qlue-ls',
  pattern = { 'sparql' },
  callback = function()
    vim.lsp.start {
      name = 'qlue-ls',
      cmd = { 'qlue-ls', 'server' },
      root_dir = vim.fn.getcwd(),
      on_attach = function(client, bufnr)
        vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, { buffer = bufnr, desc = 'LSP: ' .. '[F]ormat' })
      end,
    }
  end,
})
```

Open a `.rq` file and check that the buffer is attached to th server:

```
:checkhealth lsp
```

Configure keymaps in `on_attach` function.

# 🚀 Capabilities

## 📐 Formatting

**Status**: Full support

Formats SPARQL queries to ensure consistent and readable syntax.
Customizable options to align with preferred query styles are also implemented.

## 🩺 Diagnostics

**Status**: Partial support

** provided diagnostics**:

| Type        | Name             | Description                   |
|:------------|:-----------------|:------------------------------|
| ❌ error     | undefined prefix | a used prefix is not declared |
| ⚠️  warning | unused prefix    | a declared prefix is not used |
| ℹ️  info    | uncompacted uri  | a raw uncompacted uri is used |

## ✨ Completion

**Status**: Partial support

I split auto-completion into 3 stages:

1. Static (Keywords, constructs, ...)
2. Dynamic offline (local defined variables)
3. Dynamic online (with data from a knowledge-graph)

The implementation is in Stage 2.8.  
Dynamic online completion works!
At the moment the context is limited to the Tripple the completion is requested from. Next step is to introduce full context sensitivity.

## 🛠️ Code Actions

**Status**: Partial support

| name              | description                           | diagnostic        |
|:------------------|:--------------------------------------|:------------------|
| shorten uri       | shorten uri into compacted form       | uncompacted uri   |
| declare prefix    | declares undeclared prefix (if known) | undeclared prefix |
| shorten all uri's | shorten all uri's into compacted form |                   |
| add to result     | add variable to selected result       |                   |
| filter variable   | add filter for this variable          |                   |

# ⚙️  Configuration

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

# 🌐 use in web

If you want to connect from a web-based-editor, you can use this package as well.  
For this purpose this can be compiled to wasm and is available on [npm](https://www.npmjs.com/package/@ioannisnezis/sparql-language-server):


```shell
npm i qlue-ls
```

You will have to wrap this in a Web Worker and provide a language server client.
There will be more documentation on this in the future...

Until then you can check out the demo in ./editor/

# 🏗 Development Setup

Here is a quick guide to set this project up for development.

## Requirements

 - [rust](https://www.rust-lang.org/tools/install) >= 1.83.0
 - [wasm-pack](https://rustwasm.github.io/wasm-pack/) >= 0.13.1
 - [node & npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) >= 22.14.0 & >= 11.3.0
 - \[Optional\] [just](https://github.com/casey/just)
 - \[Optional\] [watchexec](https://github.com/watchexec/watchexec)

## Initial Setup

You will only have to do this once.

### install node dependencies

```bash
cd editor
npm install
```

### build wasm & bindings

If you have [just](https://github.com/casey/just) installed:

```bash
just build-wasm
```

If you have [make](https://wiki.ubuntuusers.de/Makefile/) installed:

```bash
make wasm
```

If you don't have [just](https://github.com/casey/just) or [make](https://wiki.ubuntuusers.de/Makefile/) installed:

**Install [just](https://github.com/casey/just)**


### link against local package

```bash
cd pkg
npm link
cd ../editor
npm link qlue-ls
```

## Run application

```bash
cd editor
npm run dev
```

Now the webapp should be running, open the browser on `localhost:5173`.

## Automatically rebuild on change

When developping the cycle is:

- Change the code
- Compile to wasm (or run tests)
- Evaluate

To avoid having to run a command each time to Compile I strongly recommend setting up a
auto runner like [watchexec](https://github.com/watchexec/watchexec).

```bash
watchexec --restart --exts rs --exts toml just build-wasm
```

or just:

```bash
just watch-and-run build-wasm
```

have fun!

# 🙏 Special Thanks

* [TJ DeVries](https://github.com/tjdevries) for the inspiration and great tutorials
* [Chris Biscardi](https://github.com/christopherbiscardi) for teaching me Rust
* [GordianDziwis](https://github.com/GordianDziwis) for providing a sparql-tree-sitter grammar

