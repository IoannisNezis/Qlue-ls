export CFLAGS_wasm32_unknown_unknown := $(shell echo "-I$(PWD)/wasm-sysroot -Wbad-function-cast -Wcast-function-type -fno-builtin")

init_dev:
	echo "Installing nodejs dependencies..."
	cd editor && npm ci
	echo "building qlue-ls wasm binary"
	wasm-pack build --release --target web
	echo "building ll-sparql-parser wasm binary"
	cd ./crates/parser/ && wasm-pack build --release --target web
	echo "linking against local packages"
	cd ./pkg/ && npm link
	cd ./crates/parser/pkg/ && npm link
	cd editor && npm link ll-sparql-parser qlue-ls
	echo "starting dev server"
	cd editor && npm run dev

wasm:
	wasm-pack build --release --target web
