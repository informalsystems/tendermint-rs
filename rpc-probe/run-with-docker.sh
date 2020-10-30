#!/bin/sh
set -e

TENDERMINT_TAG=${TENDERMINT_TAG:-latest}
TENDERMINT_IMAGE="tendermint/tendermint:${TENDERMINT_TAG}"

echo "Using Tendermint Docker image: ${TENDERMINT_IMAGE}"

cargo build

rm -rf /tmp/tendermint
mkdir -p /tmp/tendermint
docker pull "${TENDERMINT_IMAGE}"
docker run -it --rm -v "/tmp/tendermint:/tendermint" "${TENDERMINT_IMAGE}" init
docker run -d \
  --name rpc-probe-tendermint \
  --rm \
  -v "/tmp/tendermint:/tendermint" \
  -p 26657:26657 \
  "${TENDERMINT_IMAGE}" node --proxy_app=kvstore

echo "Waiting for local Docker node to come up..."
sleep 5

set +e
cargo run

docker stop rpc-probe-tendermint
