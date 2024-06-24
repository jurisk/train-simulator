bin/housekeeping.ps1

cargo build --profile wasm-release --target wasm32-unknown-unknown --package client-single-player --package networking-client --bin client_single_player --bin client_graphical
wasm-bindgen --out-name client_single_player --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_single_player.wasm
wasm-bindgen --out-name client_graphical --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_graphical.wasm

docker build --tag train-simulator-proxy --file caddy.dockerfile .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

docker tag train-simulator-proxy gcr.io/train-simulator-gcp/train-simulator-proxy
docker push gcr.io/train-simulator-gcp/train-simulator-proxy

docker build --tag train-simulator --file game-server.dockerfile .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

docker tag train-simulator gcr.io/train-simulator-gcp/train-simulator
docker push gcr.io/train-simulator-gcp/train-simulator

cd terraform
terraform fmt -recursive
terraform validate
terraform apply -auto-approve
cd ..
