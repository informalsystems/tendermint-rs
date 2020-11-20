#!/bin/sh
set -euo pipefail

# run-with-docker.sh is a helper script that sets up a Docker container with a running Tendermint node.
# The scripts input parameters are defined through environment variables. The script also accepts
# regular input parameters that are forwarded to the `rpc-probe` application.
#
# Script input variables examples:
#
# Use specific tag from official DockerHub tendermint repository:
# TENDERMINT_TAG=v0.33.0 ./run-with-docker.sh
#
# Use custom DockerHub image:
# TENDERMINT_IMAGE=informaldev/tendermint:0.34.0-stargate4 ./run-with-docker.sh
#
# Use custom local Docker image:
# DOCKER_PULL=0 TENDERMINT_IMAGE=a1c86c07867e ./run-with-docker.sh
#
# Override local tmp folder (it has to exist):
# TMP_DIR=/tmp/tendermint ./run-with-docker.sh
#
# ###
#
# rpc-probe input parameter examples:
#
# Verbose output:
# ./run-with-docker.sh -v
#
# Override output directory (default: "probe-results"):
# ./run-with-docker.sh --output "my-other-probe-results"
#
# Change request wait times when probing (default: 1000):
# ./run-with-docker.sh --request-wait 2000

DEFAULT_TENDERMINT_IMAGE="tendermint/tendermint:${TENDERMINT_TAG:-latest}"
echo "DOCKER_PULL=${DOCKER_PULL:=1}"
echo "TENDERMINT_IMAGE=${TENDERMINT_IMAGE:=$DEFAULT_TENDERMINT_IMAGE}"
echo "Application input: $@"

cargo build

echo "TMP_DIR=${TMP_DIR:=$(mktemp -d)}"

if [ "${DOCKER_PULL}" -eq "1" ]; then
  docker pull "${TENDERMINT_IMAGE}"
else
  echo "Skipping pulling of Docker image"
fi

docker run -it --rm -v "${TMP_DIR}:/tendermint" "${TENDERMINT_IMAGE}" init
docker run -d \
  --name rpc-probe-tendermint \
  --rm \
  -v "${TMP_DIR}:/tendermint" \
  -p 26657:26657 \
  "${TENDERMINT_IMAGE}" node --proxy_app=kvstore

echo "Waiting for local Docker node to come up..."
sleep 5

set +e # Try to clean up even if execution failed.
cargo run -- $@

docker stop rpc-probe-tendermint

echo "DOCKER_PULL=${DOCKER_PULL}"
echo "TENDERMINT_IMAGE=${TENDERMINT_IMAGE}"
echo "Application input: $@"
echo "TMP_DIR=${TMP_DIR}"
