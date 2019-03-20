#!/bin/bash
TMKMS_BIN=${TMKMS_BIN:-"./target/debug/tmkms"}
TMKMS_CONFIG=${TMKMS_CONFIG:-"/harness/tmkms.toml"}
HARNESS_BIN=${HARNESS_BIN:-"tm-signer-harness"}
TMHOME=${TMHOME:-"/harness"}

# Run KMS in the background
${TMKMS_BIN} start -c ${TMKMS_CONFIG} &
TMKMS_PID=$!

# Run the test harness in the foreground
${HARNESS_BIN} run \
    -addr tcp://127.0.0.1:61278 \
    -tmhome ${TMHOME}
HARNESS_EXIT_CODE=$?

# Kill the KMS, if it's still running
if ps -p ${TMKMS_PID} > /dev/null
then
    echo "Killing KMS (pid ${TMKMS_PID})"
    kill ${TMKMS_PID}
    # Wait a few seconds for KMS to die properly.
    # NOTE: This also acts as a test of the KMS listening for and properly
    # responding to the SIGTERM signal from `kill`.
    sleep 3
    # Make sure KMS has actually stopped properly now.
    if ps -p ${TMKMS_PID} > /dev/null
    then
        echo "Failed to stop KMS!"
        exit 100
    fi
else
    echo "KMS (pid ${TMKMS_PID}) already stopped, not killing"
fi

# Bubble the exit code up out of the script
echo "Harness tests exiting with code ${HARNESS_EXIT_CODE}"
exit ${HARNESS_EXIT_CODE}
