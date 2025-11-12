# 🏗 Development Setup

Here is a quick guide to set this project up for development.

## Requirements

- [rust](https://www.rust-lang.org/tools/install) >= 1.83.0
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) >= 0.13.1
- [node & npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) >= 22.14.0 & >= 11.3.0
- \[Optional\] [just](https://github.com/casey/just)
- \[Optional\] [watchexec](https://github.com/watchexec/watchexec)

## Initial Setup

In the `justfile` and `Makefile` you will find the target `init_dev`, run it:

```bash
just init_dev
```

or

```bash
make init_dev
```

It will:

- install node dependencies
- build wasm binaries (qlue-ls and ll-sparql-parser)
- run the vite dev server

The development setup uses Vite path aliases to automatically resolve local WASM packages from the `pkg/` directories during development, while production builds use the npm package versions specified in `package.json`. 


## Automatically rebuild on change

When developing the cycle is:

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
