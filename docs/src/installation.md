# Installation

There are different ways Qlue-ls can be used.  
First you could use it as a formatter, with out all the LSP stuff.  
If you want to use it as Language Server, you need to connect a editor.  
This guide covers:

- monaco
- neovim
- vscode (not yet)

If you want to use a different editor, thats not a problem.  
You just need a Language Server Client and connect it to Qlue-ls.

## Use with the Monaco editor

Installing Qlue-ls it self is straingt forward:

```bash
npm install qlue-ls
```

The hard part is setting up a language-client for monaco.

## Use with neovim

First install Qlue-ls on your machine:

```bash
cargo install qlue-ls
```

Make sure `~/.cargo/.bin/` is in your `PATH`!

## Use with Vscode

## Use as standalone formatter
