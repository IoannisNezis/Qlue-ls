test target="":
	cargo test {{target}} --bin qlue-ls

init_dev:
	echo "Installing nodejs dependencies..."
	cd editor && npm install
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

build-native:
	cargo build --release --bin qlue-ls

build-wasm profile="release" target="web":
	wasm-pack build --{{profile}} --target {{target}}

watch-and-run recipe="test":
	watchexec --restart --watch src --watch Cargo.toml -- just {{recipe}}

publish:
	wasm-pack publish
	cargo publish
	maturin publish
