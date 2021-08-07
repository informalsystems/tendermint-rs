#!/bin/bash

# release.sh will hopefully allow us to publish all of the necessary crates in
# this repo in the right order, along with a few checks and balances to try to
# avoid mistakes. It is assumed that only one person will be releasing all
# crates at the same time.
#
# For each crate, it will:
# 1. Run `cargo publish --dry-run` for that crate
# 2. List all files in the package with `cargo package --list`
# 3. Prompt the user as to whether to publish or not
# 4. Publish the package with `cargo publish` (no dry run)
#
# It has a default set of crates it will publish, which can be overridden by
# way of command line arguments:
#
#   # Release all packages, prompting for each package as to whether to publish
#   ./release.sh
#
#   # Just release the proto and tendermint crates, but nothing else
#   ./release.sh proto tendermint
#
# Once it publishes a crate, it will create a file at
# /tmp/tendermint-rs-release/${TODAY}/${CRATE}, where ${TODAY} is today's date
# and ${CRATE} is the name of the crate that was successfully published.
#
# Prior to publishing a crate, it checks whether this file is present before
# attempting to publish it. If it's present, it will ask if you really want to
# publish it again. Of course, this is pretty dumb, and doesn't cater for
# instances where multiple people could publish the crates on the same day, and
# instances where someone reboots their machine or wipes their /tmp folder
# between runs.

set -e

# A space-separated list of all the crates we want to publish, in the order in
# which they must be published. It's important to respect this order, since
# each subsequent crate depends on one or more of the preceding ones.
DEFAULT_CRATES="tendermint-proto tendermint-std-ext tendermint tendermint-abci tendermint-rpc tendermint-p2p tendermint-light-client tendermint-light-client-js tendermint-testgen"

# Allows us to override the crates we want to publish.
CRATES=${*:-${DEFAULT_CRATES}}

get_manifest_path() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.manifest_path'
}

get_local_version() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.version'
}

check_version_online() {
  curl -s "https://crates.io/api/v1/crates/${1}" | jq -r '.versions[]|select(.num == "'"${2}"'").updated_at'
}

publish() {
  echo "Publishing crate $1..."
  cargo publish --manifest-path "$(get_manifest_path "${1}")"
  echo ""
}

publish_dry_run() {
  echo "Attempting dry run of publishing crate $1..."
  cargo publish --dry-run --manifest-path "$(get_manifest_path "${1}")"
}

list_package_files() {
  cargo package --list --manifest-path "$(get_manifest_path "${1}")"
}

wait_until_available() {
  echo "Waiting for crate ${1} to become available via crates.io..."
  for retry in {1..5}; do
    sleep 5
    ONLINE_DATE="$(check_version_online "${1}" "${2}")"
    if [ -n "${ONLINE_DATE}" ]; then
      echo "Crate ${crate} is now available online"
      break
    else
      if [ "${retry}" == 5 ]; then
        echo "ERROR: Crate should have become available by now"
        exit 1
      else
        echo "Not available just yet. Waiting a few seconds..."
      fi
    fi
  done
}

echo "Attempting to publish crate(s): ${CRATES}"

for crate in ${CRATES}; do
  VERSION="$(get_local_version "${crate}")"
  ONLINE_DATE="$(check_version_online "${crate}" "${VERSION}")"
  echo "${crate} version number: ${VERSION}"
  if [ -n "${ONLINE_DATE}" ]; then
    echo "${crate} ${VERSION} has already been published at ${ONLINE_DATE}."
    read -rp "Do you want to publish again? (type YES to publish, anything else to skip) " answer
    case $answer in
      YES ) ;;
      * ) echo "Skipping"; continue;;
    esac
  fi

  publish_dry_run "${crate}"
  list_package_files "${crate}"
  echo ""
  read -rp "Are you sure you want to publish crate \"${crate}\"? (type YES to publish, anything else to exit) " answer
  case $answer in
    YES ) publish "${crate}";;
    * ) echo "Terminating"; exit;;
  esac

  wait_until_available "${crate}" "${VERSION}"
done
