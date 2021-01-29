#!/bin/bash
set -e

TENDERMINT_CONTAINER=${TENDERMINT_CONTAINER:-"abci-test-tendermint"}
TENDERMINT_IMAGE=${TENDERMINT_IMAGE:-"informaldev/tendermint:0.34.0"}
KVSTORE_RS_LOG=${KVSTORE_RS_LOG:-"/tmp/kvstore-rs.log"}

echo "Building and running kvstore-rs..."
cd ../../abci
cargo build --bin kvstore-rs --features binary,kvstore-app
cargo run --bin kvstore-rs --features binary,kvstore-app -- -v >${KVSTORE_RS_LOG} 2>&1 &
export KVSTORE_PID=$!
sleep 1

echo "Starting Tendermint..."
docker run \
  --rm \
  --name ${TENDERMINT_CONTAINER} \
  --net=host \
  --detach \
  ${TENDERMINT_IMAGE} \
  node --proxy_app tcp://127.0.0.1:26658
sleep 3

echo "Running ABCI test harness..."
set +e
cd ../tools/abci-test
cargo run -- -v

echo "Stopping Tendermint..."
docker stop ${TENDERMINT_CONTAINER}

echo "Stopping kvstore-rs"
kill ${KVSTORE_PID}

echo ""
echo "kvstore-rs logs:"
echo ""
cat ${KVSTORE_RS_LOG}

echo "Done."
