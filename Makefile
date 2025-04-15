export CFLAGS_wasm32_unknown_unknown := $(shell echo "-I$(PWD)/wasm-sysroot -Wbad-function-cast -Wcast-function-type -fno-builtin")

wasm:
	wasm-pack build --release --target web
