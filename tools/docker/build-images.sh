#!/bin/bash
set -euo pipefail

# The version of Tendermint to build into the images.
TMVERSION=${TMVERSION:-0.34.21}

echo "Building for Tendermint v${TMVERSION}..."

# Build the ABCI test harness
docker build \
    -f abci-harness/Dockerfile \
    --build-arg TMVERSION=${TMVERSION} \
    --tag informaldev/tendermint:${TMVERSION} \
    ./abci-harness/

# Build the Tendermint development image
docker build \
    -f tendermint/Dockerfile \
    --build-arg TMVERSION=${TMVERSION} \
    --tag informaldev/tendermint:${TMVERSION} \
    ./tendermint/

