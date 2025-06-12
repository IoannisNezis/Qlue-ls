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

If you plan to use qlue-ls on your machine, install it first:

## 📦 Installation

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

Here are a few common editors:

## Neovim

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

Open a `.rq` file and check that the buffer is attached to the server:

```
:checkhealth lsp
```

Configure keymaps in `on_attach` function.
