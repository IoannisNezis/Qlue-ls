export CFLAGS_wasm32_unknown_unknown := `echo "-I$(pwd)/wasm-sysroot -Wbad-function-cast -Wcast-function-type -fno-builtin"`

test target="":
	cargo test {{target}} --bin qlue-ls

start-monaco-editor:
	cd editor && npm install && npm run dev

build-native:
	hyprctl notify 1 2000 0 starting build...
	cargo build --release --bin qlue-ls
	hyprctl notify 1 1000 0 build done

build-wasm profile="release" target="web":
	hyprctl notify 1 2000 0 starting build...
	wasm-pack build --{{profile}} --target {{target}}
	hyprctl notify 1 1000 0 build done

watch-and-run recipe="test":
	watchexec --restart --exts rs --exts toml just {{recipe}}

publish:
	wasm-pack publish
	maturin publish
	cargo publish
