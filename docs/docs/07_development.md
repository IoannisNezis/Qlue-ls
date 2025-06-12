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
- build wasm binaries
- link against local packages
- run the vite dev server

If you don't have [just](https://github.com/casey/just) or [make](https://wiki.ubuntuusers.de/Makefile/) installed:

**Install [just](https://github.com/casey/just)**


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
