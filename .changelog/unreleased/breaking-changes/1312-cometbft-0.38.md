- [`tendermint`] Adaptations for CometFBT 0.38
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)):
  * Define `consensus::params::AbciParams` struct, add the `abci` field of this
    type to `consensus::Params` to represent the protobuf additions.
  * Represent vote extensions in the `Vote` domain type.
  * Remove the fields from the `CheckTx` and `Commit` domain types that are
    no longer present in CometBFT 0.38.
- [`tendermint`] Define version-specific categorized request/response enums:
  `ConsensusRequest`, `MempoolRequest`, `InfoRequest`, `ShapshotRequest`,
  `ConsensusResponse`, `MempoolResponse`, `InfoResponse`, `ShapshotResponse`,
  in each of the `v0_*::abci` modules, so that the variants are trimmed to the
  requests/responses used by the respective protocol version.
  Reexport the types from `v0_38::abci` as continuation for these names in the
  `abci` module
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)).
- [`tendermint`] Rename `Signature::to_bytes` to `Signature::into_bytes`
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)).
- [`tendermint-abci`] Update the `Application` interface to CometBFT 0.38
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312))
