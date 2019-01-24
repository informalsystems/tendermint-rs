#!/bin/bash
TMKMS_BIN=${TMKMS_BIN:-"./target/debug/tmkms"}
TMKMS_CONFIG=${TMKMS_CONFIG:-"/remote_val_harness/tmkms.toml"}
REMOTE_VAL_HARNESS_BIN=${REMOTE_VAL_HARNESS_BIN:-"remote_val_harness"}
TMHOME=${TMHOME:-"/remote_val_harness"}

# Run KMS in the background
${TMKMS_BIN} start -c ${TMKMS_CONFIG} &
TMKMS_PID=$!

# Run the test harness in the foreground
${REMOTE_VAL_HARNESS_BIN} run \
    --addr tcp://127.0.0.1:61278 \
    --genesis-file ${TMHOME}/config/genesis.json \
    --key-file ${TMHOME}/config/priv_validator_key.json \
    --state-file ${TMHOME}/data/priv_validator_state.json
HARNESS_EXIT_CODE=$?

# Kill the KMS, if it's still running
if ps -p ${TMKMS_PID} > /dev/null
then
    echo "Killing KMS (pid ${TMKMS_PID})"
    kill ${TMKMS_PID}
else
    echo "KMS (pid ${TMKMS_PID}) already stopped, not killing"
fi

# Bubble the exit code up out of the script
echo "Harness tests exiting with code ${HARNESS_EXIT_CODE}"
exit ${HARNESS_EXIT_CODE}
