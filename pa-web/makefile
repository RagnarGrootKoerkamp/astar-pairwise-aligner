# ========== WASM =========
wasm:
	wasm-pack build --debug --target web
wasm_prod:
	wasm-pack build --release --target web

run: wasm
	python3 -m http.server
