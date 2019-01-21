#!/bin/bash

PWD=`pwd`
TMHOME=${TMHOME:-${PWD}}
OUTPUT_PATH=${OUTPUT_PATH:-${PWD}}
GENESIS_FILE=${GENESIS_FILE:-${TMHOME}/config/genesis.json}
SIGNING_KEY=${SIGNING_KEY:-${OUTPUT_PATH}/signing.key}
SECRET_KEY=${SECRET_KEY:-${OUTPUT_PATH}/secret_connection.key}
OUTPUT_FILE=${OUTPUT_FILE:-${OUTPUT_PATH}/tmkms.toml}
VALIDATOR_ADDR=${VALIDATOR_ADDR:-"tcp://127.0.0.1:61278"}
CFG_TEMPLATE=$(cat <<-EOF
[[validator]]
addr = "VALIDATOR_ADDR"
chain_id = "CHAIN_ID"
reconnect = true # true is the default
secret_key = "SECRET_KEY"

[[providers.softsign]]
id = "CHAIN_ID"
path = "SIGNING_KEY"
EOF
)

# First extract the chain ID from the genesis file
CHAIN_ID_SED_EXPR='s/[ ]*"chain_id":[ ]*"\([^"]*\)".*/\1/'
CHAIN_ID=`grep '"chain_id"' ${GENESIS_FILE} | sed "${CHAIN_ID_SED_EXPR}"`

# Now generate the tmkms.toml file
echo "${CFG_TEMPLATE}" | \
    sed "s|CHAIN_ID|${CHAIN_ID}|g" | \
    sed "s|VALIDATOR_ADDR|${VALIDATOR_ADDR}|g" | \
    sed "s|SECRET_KEY|${SECRET_KEY}|g" | \
    sed "s|SIGNING_KEY|${SIGNING_KEY}|g" > ${OUTPUT_FILE}

echo "Wrote ${OUTPUT_FILE}"
