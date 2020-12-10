#!/usr/bin/env sh
# This script is a template on how to generate gaiad config files (folder "n0") for a one-node validator.
# Provide your own gaiad binary in the GAIAD environment variable.
# Get sconfig from here: https://github.com/freshautomations/sconfig/releases
# Run example: GAIAD=./gaiad_macos SCONFIG=/usr/local/bin/sconfig ./create-config.sh

set -eu

# Requirements for this specific script
echo "gaiad binary: ${GAIAD:=./gaiad}" && test -x "${GAIAD}"
echo "jq binary: ${JQ:=$(which jq)}" && test -x "${JQ}"
echo "sconfig binary: ${SCONFIG:=$(which sconfig)}" && test -x "${SCONFIG}"
test -d gentxs && echo "** gentxs folder already exists, please remove it, quitting" && false
test -d n0 && echo "** n0 folder already exists, please remove it, quitting" && false

# Create "n0" validator config
# Warning: `--node-dir-prefix ""` makes gaiad panic.
"${GAIAD}" testnet \
  --chain-id dockerchain \
  --keyring-backend test \
  --minimum-gas-prices 1.0uatom \
  --node-daemon-home . \
  --node-dir-prefix n \
  --output-dir . \
  --v 1
mv gentxs n0

# Make some modification in the validator configuration
"${SCONFIG}" -s n0/config/config.toml \
  moniker=dockernode \
  consensus.timeout_commit=500ms \
  p2p.addr_book_strict=false \
  instrumentation.prometheus=true
"${SCONFIG}" -s n0/config/genesis.json \
  consensus_params.block.time_iota_ms=500

# Create "c0" client wallet
C0_ADDRESS="$("${GAIAD}" keys add c0 \
  --keyring-backend test \
  --keyring-dir n0 \
  --output json | "${JQ}" -r '.address' )"
echo "Created client address: ${C0_ADDRESS}"

# Make the "c0" wallet a millionaire in the genesis file.
"${GAIAD}" add-genesis-account \
  "${C0_ADDRESS}" \
  1000000000000uatom,1000000000000stake,1000000000000n0token \
  --home n0
echo "Done."
