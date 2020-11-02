#!/bin/sh
set -e

TENDERMINT_TAG=${TENDERMINT_TAG:-latest}
DOCKER_PULL=${DOCKER_PULL:-1}
VERBOSE=${VERBOSE:-0}
OUTPUT=${OUTPUT:-"probe-results"}
REQUEST_WAIT=${REQUEST_WAIT:-1000}

TENDERMINT_IMAGE="tendermint/tendermint:${TENDERMINT_TAG}"
FLAGS="--output ${OUTPUT} --request-wait ${REQUEST_WAIT}"
if [ "${VERBOSE}" -eq "1" ]; then
  FLAGS="${FLAGS} -v"
fi

echo "Using Tendermint Docker image: ${TENDERMINT_IMAGE}"

cargo build

rm -rf /tmp/tendermint
mkdir -p /tmp/tendermint

if [ "${DOCKER_PULL}" -eq "1" ]; then
  docker pull "${TENDERMINT_IMAGE}"
else
  echo "Skipping pulling of Docker image"
fi

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
cargo run -- ${FLAGS}

docker stop rpc-probe-tendermint
