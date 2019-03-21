#!/bin/bash

PWD=`pwd`
TMHOME=${TMHOME:-${PWD}}
OUTPUT_PATH=${OUTPUT_PATH:-${PWD}}
GENESIS_FILE=${GENESIS_FILE:-${TMHOME}/config/genesis.json}
SIGNING_KEY=${SIGNING_KEY:-${OUTPUT_PATH}/signing.key}
SECRET_KEY=${SECRET_KEY:-${OUTPUT_PATH}/secret_connection.key}
OUTPUT_FILE=${OUTPUT_FILE:-${OUTPUT_PATH}/tmkms.toml}
#TODO: Restore once https://github.com/tendermint/tendermint/issues/3105 is resolved
#VALIDATOR_ID=${VALIDATOR_ID:-"f88883b673fc69d7869cab098de3bafc2ff76eb8"}
#VALIDATOR_ADDR=${VALIDATOR_ADDR:-"tcp://${VALIDATOR_ID}@127.0.0.1:61278"}
VALIDATOR_ADDR=${VALIDATOR_ADDR:-"tcp://127.0.0.1:61278"}
CFG_TEMPLATE=$(cat <<-EOF
# Information about Tenderment blockchain networks this KMS services
[[chain]]
id = "CHAIN_ID"
key_format = { type = "bech32", account_key_prefix = "cosmospub", consensus_key_prefix = "cosmosvalconspub" }

[[validator]]
addr = "VALIDATOR_ADDR"
chain_id = "CHAIN_ID"
reconnect = true # true is the default
secret_key = "SECRET_KEY"

[[providers.softsign]]
chain_ids = ["CHAIN_ID"]
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
