Remove-Item static/assets -Recurse -Force
Copy-Item assets -Destination static/assets -Recurse -Force

cargo build --profile wasm-release --target wasm32-unknown-unknown --package client-single-player --package networking-client --bin client_single_player --bin client_graphical
wasm-bindgen --out-name client_single_player --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_single_player.wasm
wasm-bindgen --out-name client_graphical --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_graphical.wasm