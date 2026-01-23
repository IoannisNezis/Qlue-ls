# Getting started

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

You can use Qlue-ls with any tool that has a lsp-client.

## Local Installation

Qlue-ls is available on [crate.io](https://crates.io/crates/qlue-ls):

```shell
cargo install qlue-ls
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

## Web Usage

If you want to connect from a web-based-editor, you can use this package as well.
For this purpose this can be compiled to wasm and is available on [npm](https://www.npmjs.com/package/@ioannisnezis/sparql-language-server):

```shell
npm i qlue-ls
```

You will have to wrap this in a Web Worker and provide a language server client.
There will be more documentation on this in the future...

Until then you can check out the demo in ./editor/

## Editor Setup

Here are a few common editors:

### Neovim

After you installed the language server, add this to your `init.lua`:

```lua
vim.lsp.config('qlue-ls',{
  filetypes = {'sparql'},
  cmd = { 'qlue-ls', 'server' },
  root_dir = vim.fn.getcwd(),
  -- Set any keymaps you want to use with Qlue-ls
  on_attach = function(client, bufnr)
    vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, { buffer = bufnr, desc = 'LSP: ' .. '[F]ormat' })
  end,
}
)

vim.lsp.enable({'qlue-ls'})
```

With the above setup, Qlue-ls will load [configuration](/03_configuration) from a file called `qlue-ls.{yml, toml}` in the working
directory neovim is launched from.  All file formats supported by the config create are valid.

Open a `.rq` file and check that the buffer is attached to the server:

```
:checkhealth lsp
```
