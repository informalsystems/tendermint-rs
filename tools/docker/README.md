# Docker images

This folder contains `Dockerfile` configurations that are used during
development and testing.

The folders are named `<name>-<version>`, like `tendermint-0.34.0`.

The created images are uploaded to DockerHub, under the informaldev
organization. For example: `informaldev/tendermint:0.34.0`.

## tendermint

This image is used during CI testing in the tendermint-rs crate and it can be
used during fixture creation with `rpc-probe`.  It tests compatibility with the
Tendermint Go implementation.  It is a GitHub Actions "Services"-compatible
image: a standalone image that can run on its own. It can create its own
configuration if one was not provided. This ensures that the configuration file
is always compatible with the Tendermint version built into it.

## gaiad

This image will be used for `rpc-probe`, to generate fixtures for CI testing
from a gaiad node.

Contrary to the `tendermint` image, the configuration here is pre-created so the
genesis file can be populated with additional wallets. The corresponding private
keys are also saved into a test keyring.

All the configuration is in the `n0` folder. Two wallets are created `c0` and
`c1` (the validator's key is `n0`.) Both wallets have `uatom`, `stake` and
`n0token` added.

Both wallets have an initial signed transaction created for easier population of
the network before testing. These transactions will send uatom tokens from c0 ->
c1 and vice versa. They are both signed as `sequence 0` in the wallet, so they
can only be executed as the first transaction of the corresponding wallet.

# abci-harness

This image is used during CI testing in the abci-rs crate.  It tests
compatibility with the Tendermint Go implementation.  It derives from the
Tendermint Docker image above, but it expects a volume attached at `/abci` that
contains the ABCI application to be tested. The name of the ABCI application is
`kvstore-rs` by default. This can be changed by setting the `ABCI_APP`
environment variable.

The image will fire up a Tendermint node (auto-creating the configuration) and
then execute the ABCI application from the attached volume. It logs the
Tendermint node log into kvstore-rs.tendermint and the ABCI application log into
kvstore-rs.log on the attached volume.

This image has both the `muslc` and `glibc` libraries installed for easy testing
of dynamically linked binaries.

Example:
```bash
docker run -it --rm -v $PWD/target/debug:/abci -p 26657:26657 informaldev/abci-harness:0.34.0
```

The image version reflects the Tendermint Go binary version.
