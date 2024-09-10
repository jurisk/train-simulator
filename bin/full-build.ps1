bin/housekeeping.ps1

bin/wasm-build.ps1

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
