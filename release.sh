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

ARGS="$@"
TODAY=$(date +%Y-%m-%d)
TMP_RELEASE_FOLDER="/tmp/tendermint-rs-release/${TODAY}"
mkdir -p "${TMP_RELEASE_FOLDER}"

# A space-separated list of all the crates we want to publish, in the order in
# which they must be published. It's important to respect this order, since
# each subsequent crate depends on one or more of the preceding ones.
DEFAULT_CRATES="proto tendermint rpc light-client light-node testgen"

# Allows us to override the crates we want to publish.
CRATES=${ARGS:-${DEFAULT_CRATES}}
read -ra CRATES_ARR <<< "${CRATES}"

publish() {
  echo "Publishing crate $1..."
  cargo publish --manifest-path "$1/Cargo.toml"
  echo ""

  # Remember that we've published this crate today
  touch "${TMP_RELEASE_FOLDER}/$1"
}

publish_dry_run() {
  echo "Attempting dry run of publishing crate $1..."
  cargo publish --dry-run --manifest-path "$1/Cargo.toml"
}

list_package_files() {
  cargo package --list --manifest-path "$1/Cargo.toml"
}

echo "Attempting to publish crate(s): ${CRATES}"

for crate in "${CRATES_ARR[@]}"; do
  if [ -f "${TMP_RELEASE_FOLDER}/${crate}" ]; then
    echo "Crate \"${crate}\" has already been published today."
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
    YES ) publish "${crate}"; break;;
    * ) echo "Terminating"; exit;;
  esac
done
