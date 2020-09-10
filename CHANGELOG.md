## Unreleased

- Add spec for the light client attack evidence handling ([#526])
- Return RFC6962 hash for empty merkle tree ([#498])
- The `tendermint`, `tendermint-rpc`, and `tendermint-light-client` crates now compile to WASM on the `wasm32-unknown-unknown` and `wasm32-wasi` targets ([#463])
- Implement protobuf encoding/decoding of Tendermint Proto types ([#504])
- Separate protobuf types from Rust domain types using the DomainType trait ([#535])

[#526]: https://github.com/informalsystems/tendermint-rs/issues/526
[#498]: https://github.com/informalsystems/tendermint-rs/issues/498
[#463]: https://github.com/informalsystems/tendermint-rs/issues/463
[#504]: https://github.com/informalsystems/tendermint-rs/issues/504
[#535]: https://github.com/informalsystems/tendermint-rs/issues/535

## v0.16.0

*Aug 31, 2020*

This release is the first release of the [testgen][testgen-dir] utility, 
a generator for Tendermint types for unit and integration tests and for model-based testing. 
It is a utility for producing tendermint datastructures from minimal input, targeted for testing.

The release also contains various Rust API-breaking changes. It remains compatible with v0.33 of Tendermint Core.

 ⚠️ ️Deprecation warning ⚠️ : The `lite` module was removed. Please take a look at the [light-client][light-client-dir] crate.

### BREAKING CHANGES:

- [repo] CHANGES.md renamed to CHANGELOG.md
- [tendermint] Eliminate use of `signatory` wrapper crate in favour of underlying `ed25519-dalek` and `k256` crates. `ed25519-dalek` is now v1.0 and `k256` provides a pure Rust implementation of secp256k1 rather than wrapping the C library ([#522])
- [tendermint] Remove `lite` and `lite_impl` modules. See the new `light-client`
  crate ([#500])

### FEATURES:

- [tendermint/proto] A tendermint-proto crate was created that contains the Rust structs for protobuf,
preparing for compatibility with Tendermint Core v0.34 ([#508])
- [tendermint/proto-compiler] A tendermint-proto-compiler crate was created that generates the tendermint-proto structs from the Tendermint Core Protobuf definitions.
- [testgen] Introduce the `testgen` crate for generating Tendermint types from
  minimal input ([#468])

### IMPROVEMENTS:

- [light-client] Use the `testgen` for generating tests
- [light-client] Use primary error as context of `NoWitnessLeft` error ([#477])
- [repo] Various improvements to documentation and crate structure
- [repo] Add CONTRIBUTING.md document ([#470])
- [specs] Updates to fork detection English spec for evidence handling in
  Tendermint and IBC ([#479])
- [specs] Model checking results and updates for the fast sync TLA+ spec ([#466])

### BUG FIXES:

- [light-client] Fix to reject headers from the future ([#474])

[light-client-dir]: https://github.com/informalsystems/tendermint-rs/tree/master/light-client
[testgen-dir]: https://github.com/informalsystems/tendermint-rs/tree/master/testgen

[#466]: https://github.com/informalsystems/tendermint-rs/pull/466
[#468]: https://github.com/informalsystems/tendermint-rs/pull/468
[#470]: https://github.com/informalsystems/tendermint-rs/pull/470
[#474]: https://github.com/informalsystems/tendermint-rs/pull/474
[#477]: https://github.com/informalsystems/tendermint-rs/pull/477
[#479]: https://github.com/informalsystems/tendermint-rs/pull/479
[#500]: https://github.com/informalsystems/tendermint-rs/pull/500
[#508]: https://github.com/informalsystems/tendermint-rs/pull/508
[#522]: https://github.com/informalsystems/tendermint-rs/pull/522

## v0.15.0

*July 17, 2020*

This release is the first official release of the revamped [light-client][light-client-dir] library and the [light-node][light-node-dir] command-line interface.
Together they provide a complete Tendermint light client implementation that performs squential and skipping verification
and attempts to detect forks across its peers. Complete TLA+ specifications for light client verification are included,
along with work-in-progress specs for fork detection. The implementation is compatible with v0.33 of Tendermint Core.

Note that both the [light-client][light-client-dir]  and [light-node][light-node-dir] crates are to be considered experimental software that will still undergo a 
lot of improvements and iterations. The goal of releasing an early version of our Light Client is to make it accessible, to get people use it, and to receive feedback.

An overview of the current design of the light client is provided in [ADR-006]
and [ADR-007].


 ⚠️ ️Deprecation warning ⚠️ : This might be the last release containing the [lite][lite-dir] module. Please take a look at the [light-client][light-client-dir] crate.

### BREAKING CHANGES:

- [repo] make secp256k1 dependency optional ([#441])

### FEATURES:

- [light-client] Rewrite and expansion of `lite`, the prior light client
  verification module, into a new fully-featured `light-client` crate. The crate provides a db, 
  functions for complete light client verification, peer management, fork detection, and evidence reporting,
  along with extensive testing. Components are composed via a `Supervisor`, which is run in its own thread, 
  and exposes a Handle trait to broker access to underlying state and
  functionality. See the [light-client][light-client-dir] crate for details.
- [light-node] New binary crate with CLI for running the light client as a daemon,
  complete with an rpc server for querying the latest state of the light node
  while it syncs with the blockchain. See the [light-node][light-node-dir] crate
  for details.

### BUG FIXES:

- [tendermint/validator] Sort validators by address on deserialization ([#410])
- [tendermint/validator] Fix deserializing Update struct when power field is 0
  ([#451])
- [tendermint/abci] Fix DeliverTx response deserialization issues with
  gasWanted, gasUsed, and data fields ([#432])
- [tendermint/lite_impl] Fix header.hash for height 1 ([#438])

[#410]: https://github.com/informalsystems/tendermint-rs/pull/410
[#432]: https://github.com/informalsystems/tendermint-rs/pull/432
[#438]: https://github.com/informalsystems/tendermint-rs/pull/438
[#441]: https://github.com/informalsystems/tendermint-rs/pull/441
[#451]: https://github.com/informalsystems/tendermint-rs/pull/451

[ADR-006]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/architecture/adr-006-light-client-refactor.md
[ADR-007]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/architecture/adr-007-light-client-supervisor-ergonomics.md

[lite-dir]: ./tendermint/src/lite
[light-client-dir]: ./light-client
[light-node-dir]: ./light-node/

## [0.14.1] (2020-06-23)

- Update `prost-amino`/`prost-amino-derive` to v0.6 ([#367])

[#367]: https://github.com/informalsystems/tendermint-rs/issues/367
[0.14.1]: https://github.com/informalsystems/tendermint-rs/pull/368

## [0.14.0] (2020-06-19)

This release mainly targets compatibility with Tendermint [v0.33.x] but contains a lot of smaller improvements regarding testing and (de)serialization.
Also noteworthy is that the rpc module was broken out into a separate crate ([tendermint-rpc]).

⚠️ ️Deprecation warning ⚠️ : This might be that last release containing the [lite] module.
It will be replaced with the [light-client][light-client-dir] crate (soon).

CommitSig:
- Refactored CommitSig into a more Rust-friendly enum. ([#247])
- Added CommitSig compatibility code to Absent vote ([#260])
- Added CommitSig timestamp zero-check compatibility code ([#259])

Testing:
- Configure integration test against latest tendermint-go to continue on error ([#304])
- Add integration test to track tendermint-go v0.33.5 ([#304])
- Remove test for hard-coded version in `abci_info` ([#304])

Serialization:
- Refactor serializers library to use modules, give a nicer annotation to structs and separated into its own folder. ([#247])
- Added nullable Vec<u8> serialization ([#247])
- Moved/created tests for serialization in the same library and locked library to local crate ([#263])
- Made serialization tests symmetric ([#261])

RPC:
- Tendermint-Go v0.33 compatibility ([#184])
  - `abci_info`, `abci_query`, `block_results`, `genesis` structs
  - serialization/deserialization fixes
  - Updated/fixed integration tests
- Move into its own crate ([#338])
  - Feature guard `rpc::client` (makes networking an optional dependency) ([#343])

CI:
- Moved to GitHub Actions ([#120])
- Updated crates.io badges ([#120])
- Enabled integration tests in CI with Tendermint-Go node service ([#120])
- Exclude changes in docs folder to trigger CI execution ([#309])

[#120]: https://github.com/informalsystems/tendermint-rs/issues/120
[#184]: https://github.com/informalsystems/tendermint-rs/issues/184
[#247]: https://github.com/informalsystems/tendermint-rs/issues/247
[#259]: https://github.com/informalsystems/tendermint-rs/issues/259
[#260]: https://github.com/informalsystems/tendermint-rs/issues/260
[#261]: https://github.com/informalsystems/tendermint-rs/issues/261
[#263]: https://github.com/informalsystems/tendermint-rs/issues/263
[#304]: https://github.com/informalsystems/tendermint-rs/issues/304
[#309]: https://github.com/informalsystems/tendermint-rs/issues/309
[#338]: https://github.com/informalsystems/tendermint-rs/pull/338
[#343]: https://github.com/informalsystems/tendermint-rs/pull/343

[0.14.0]: https://github.com/informalsystems/tendermint-rs/pull/347
[v0.33.x]: https://github.com/tendermint/tendermint/blob/v0.33.5/CHANGELOG.md#v0335
[tendermint-rpc]: https://github.com/informalsystems/tendermint-rs/tree/master/rpc#tendermint-rpc
[lite]: https://github.com/informalsystems/tendermint-rs/tree/master/tendermint/src/lite
[light-client-dir]: https://github.com/informalsystems/tendermint-rs/tree/master/light-client

## [0.13.0] (2020-04-20)

Dependencies:

- Update `signatory` requirement to v0.19 ([#227])

[0.13.0]: https://github.com/informalsystems/tendermint-rs/pull/228
[#227]: https://github.com/informalsystems/tendermint-rs/pull/227

## [0.12.0] (2020-04-17)

Dependencies
- Update to bytes `0.5` and amino_rs `0.5`.
- Tokens for amino_rs are now fully non-conflicting with prost. Allowing both to be used together
- Made RPC type values optional for full compatibility with tendermint-go@v0.32: `abci_info`, `abci_query` [#120]
- JSON ID is JSON specification compatible and accepts int, string or null - [#88]

## [0.11.0] (2019-12-11)

This is the first release since this repository was split off
from the [KMS](https://github.com/tendermint/kms) repo a few months
ago and contains more than the usual number of changes.
As the new repository matures we will be working towards a more robust
release cycle.

This release also contains a first draft of the Tendermint Light Client :).

The changes are organized in sections for better readability.

Organizational Changes:

- Reorganized the crate into a workspace with a `tendermint` crate ([#30])
- Remove all optional compilation ([#16])
- Started using CircleCI for continuous integration ([#15])
- Fix clippy lints ([#40], [#55])

RPC Changes:

- Fix `/commit` endpoint to actually include the commit data ([#42])
- Use async/await for the rpc client ([#85])

Type Changes:

- Add `Default` trait impls and some other utilities to data types ([#64])
- Fix transaction hash length to be 32-bytes ([#14])
- Rename `LastCommit` to `Commit` ([#42])
- Fix genesis file to include `validators` field ([#65])
- Change `max_gas` from `u64` to `i64` ([#61])
- Allow `Height` to be `0` ([#77])

ABCI Changes:

- Include `AbciQuery` in the `Method` enum ([#AbciQueryMethodEnum])
- Fix deserializing ABCI Code field ([#13])
- Fix ABCI data field to allow lower case hex encodings ([#17])
- Fix `/abci_query` endpoint to take input `data` as hex and return `key`
  and `value` in the response as base64 ([#77])

Light Client:

- Introduce validator `Set` type and compute Merkle root ([#6])
- First draft implementation of logic for the light client ([#31, #36])

- Dependency Changes:

- Remove `secret_connection` and `ring` as dependencies (moved to KMS repo)
  ([#60])
- `tai64` from `2` to `3` ([#22])
- `zeroize` from `0.9` to `1.1` ([#74, #89])
- `hyper` from `0.10` to `0.13` ([#85])
- `signatory` from `0.12` to `0.17` ([#89])
- `subtle-encoding` from `0.3` to `0.5` ([#47])
- `uuid` from `0.7` to `0.8` ([#91])
- replace `rand_os` with `getrandom` ([#90])

## [0.10.0] (2019-07-30)

This release is tested against [tendermint v0.31] and known to be compatible
with [tendermint v0.32] aside from one known issue impacting RPC ([#286]).

- Fix inclusive range incompatibility affecting Rust nightly ([#326])
- Derive Eq/Ord for (transitive) status types ([#324])
- Add `TendermintConfig::load_node_key` ([#315])
- Add `TendermintConfig::load_genesis_file` ([#312])
- Add `TendermintConfig` and `Error(Kind)` types ([#298])
- Support `/abci_query` RPC endpoint ([#296])
- Implement the Tendermint (RFC6962) Merkle tree ([#292])
- Support `account::Id` generation from ed25519 pubkeys ([#291])

## [0.9.0] (2019-06-24)

This release is compatible with [tendermint v0.31]

- Reject low order points in Secret Connection handshake ([#279])
- Add `RemoteErrorCode` enum ([#272])
- Add `msg_type()` accessor for signature types ([#271])

## [0.8.0] (2019-06-20)

This release is compatible with [tendermint v0.31]

- `/block_results` RPC endpoint and related types ([#267], [#268])
- Upgrade to Signatory v0.12 ([#259])

## [0.7.0] (2019-04-24)

This release is compatible with [tendermint v0.31]

- Initial JSONRPC over HTTP client + `/broadcast_tx_*` endpoints ([#243])
- Initial RPC support ([#235])
- Disallow a block height of 0 ([#234])

## [0.6.0] (2019-04-16)

This release is compatible with [tendermint v0.31]

- Add `tendermint::Address`, `tendermint::account::Id`, `tendermint::Moniker`,
  and improve `serde` serializer support ([#228]).

## [0.5.0] (2019-03-13)

This release is compatible with [tendermint v0.30]

- Rename `SecretConnectionKey` to `secret_connection::PublicKey`, add
  `secret_connection::PeerId` ([#219])
- Move `ConsensusState` under `chain::state` ([#205])

## 0.4.0 (N/A)

- Skipped to synchronize versions with `tmkms`

## 0.3.0 (2019-03-05)

- Support for secp256k1 keys ([#181])

## 0.2.0 (2019-01-23)

This release is compatible with [tendermint v0.29]

- Update to x25519-dalek v0.4.4 (#158)
- Consistent ordering of `BlockID` and `Timestamps` in vote and proposal messages (#159)
- Remove `PoisonPillMsg` previously used to shut-down the kms (#162)

## 0.1.5 (2019-01-18)

This release is compatible with [tendermint v0.28]

- Split `PubKeyMsg` into `PubKeyRequest` and `PubKeyResponse` (#141)
- Migrate to Rust 2018 edition (#138)

## 0.1.4 (2018-12-02)

- Allow empty BlockIds in validation method (#131)

## 0.1.3 (2018-12-01)

- Prefix bech32 encoding of consensus keys with amino prefix (#128)

## 0.1.2 (2018-11-27)

- Update to subtle-encoding v0.3 (#124)
- Introduce same validation logic as Tendermint (#110)
- Remove heartbeat (#105)

## 0.1.1 (2018-11-20)

- Minor clarifications/fixes (#103)

## 0.1.0 (2018-11-13)

- Initial release

[0.10.0]: https://github.com/tendermint/kms/pull/328
[tendermint v0.32]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0320
[#326]: https://github.com/tendermint/kms/pull/326
[#324]: https://github.com/tendermint/kms/pull/324
[#315]: https://github.com/tendermint/kms/pull/315
[#312]: https://github.com/tendermint/kms/pull/312
[#298]: https://github.com/tendermint/kms/pull/298
[#296]: https://github.com/tendermint/kms/pull/296
[#292]: https://github.com/tendermint/kms/pull/292
[#291]: https://github.com/tendermint/kms/pull/291
[#286]: https://github.com/tendermint/kms/pull/286
[0.9.0]: https://github.com/tendermint/kms/pull/280
[#279]: https://github.com/tendermint/kms/pull/279
[#272]: https://github.com/tendermint/kms/pull/272
[#271]: https://github.com/tendermint/kms/pull/271
[0.8.0]: https://github.com/tendermint/kms/pull/269
[#268]: https://github.com/tendermint/kms/pull/268
[#267]: https://github.com/tendermint/kms/pull/267
[#259]: https://github.com/tendermint/kms/pull/259
[0.7.0]: https://github.com/tendermint/kms/pull/247
[#243]: https://github.com/tendermint/kms/pull/243
[#235]: https://github.com/tendermint/kms/pull/235
[#234]: https://github.com/tendermint/kms/pull/234
[0.6.0]: https://github.com/tendermint/kms/pull/229
[tendermint v0.31]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0310
[#228]: https://github.com/tendermint/kms/pull/228
[0.5.0]: https://github.com/tendermint/kms/pull/220
[tendermint v0.30]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0300
[#219]: https://github.com/tendermint/kms/pull/219
[#205]: https://github.com/tendermint/kms/pull/219
[#181]: https://github.com/tendermint/kms/pull/181
[tendermint v0.29]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0290
[tendermint v0.28]: https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md#v0280
[#30]: https://github.com/interchainio/tendermint-rs/pull/30
[#16]: https://github.com/interchainio/tendermint-rs/pull/16
[#15]: https://github.com/interchainio/tendermint-rs/pull/15
[#40]: https://github.com/interchainio/tendermint-rs/pull/40
[#55]: https://github.com/interchainio/tendermint-rs/pull/55
[#85]: https://github.com/interchainio/tendermint-rs/pull/85
[#64]: https://github.com/interchainio/tendermint-rs/pull/64
[#14]: https://github.com/interchainio/tendermint-rs/pull/14
[#42]: https://github.com/interchainio/tendermint-rs/pull/42
[#65]: https://github.com/interchainio/tendermint-rs/pull/65
[#61]: https://github.com/interchainio/tendermint-rs/pull/61
[#AbciQueryMethodEnum]:
https://github.com/interchainio/tendermint-rs/commit/566dfb6a9ef9659a504b43fb8ccb5c5e7969e3a0
[#13]: https://github.com/interchainio/tendermint-rs/pull/13
[#17]: https://github.com/interchainio/tendermint-rs/pull/17
[#77]: https://github.com/interchainio/tendermint-rs/pull/77
[#6]: https://github.com/interchainio/tendermint-rs/pull/6
[#31]: https://github.com/interchainio/tendermint-rs/pull/31
[#36]: https://github.com/interchainio/tendermint-rs/pull/36
[#60]: https://github.com/interchainio/tendermint-rs/pull/60
[#22]: https://github.com/interchainio/tendermint-rs/pull/22
[#74]: https://github.com/interchainio/tendermint-rs/pull/74
[#89]: https://github.com/interchainio/tendermint-rs/pull/89
[#47]: https://github.com/interchainio/tendermint-rs/pull/47
[#90]: https://github.com/interchainio/tendermint-rs/pull/90
[#83]: https://github.com/interchainio/tendermint-rs/pull/83
[#91]: https://github.com/interchainio/tendermint-rs/pull/91
