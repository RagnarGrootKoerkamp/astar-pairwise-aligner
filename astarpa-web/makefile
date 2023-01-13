# ========== WASM =========
wasm:
	wasm-pack build --debug --target web --no-default-features --features wasm
wasm_prod:
	wasm-pack build --release --target web --no-default-features --features wasm

run: wasm
	python3 -m http.server
