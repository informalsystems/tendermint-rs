- [`tendermint-proto`] Generate prost bindings for CometBFT 0.38
  under the `tendermint::v0_38` module
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312))
- [`tendermint`] Support for CometBFT 0.38:
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)):
  * Add conversions to and from `tendermint::v0_38` protobuf
    types generated in [`tendermint-proto`].
  * Add request and response enums under `v0_38::abci` to enumerate all requests
    and responses appropriate for CometBFT version 0.38.
  * Add request and response types under `abci` to represent the requests
    and responses new to ABCI++ 2.0 in CometBFT version 0.38. The names are
    `ExtendVote`, `FinalizeBlock`, `VerifyVoteExtension`.