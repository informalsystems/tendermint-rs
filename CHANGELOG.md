# CHANGELOG

## v0.33.0

This release adds support for CometBFT 0.38 protocols.
In the response to the `sync_info` RPC endpoint, the data of the earliest block
are exposed to the client.

### BREAKING CHANGES

- `[tendermint]` Adaptations for CometFBT 0.38
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)):
  * Define `consensus::params::AbciParams` struct, add the `abci` field of this
    type to `consensus::Params` to represent the protobuf additions.
  * Change the `abci::Request` and `abci::Response` reexports to use the
    enums defined in `v0_38`.
- `[tendermint]` Define version-specific categorized request/response enums:
  `ConsensusRequest`, `MempoolRequest`, `InfoRequest`, `ShapshotRequest`,
  `ConsensusResponse`, `MempoolResponse`, `InfoResponse`, `ShapshotResponse`,
  in each of the `v0_*::abci` modules, so that the variants are trimmed to the
  requests/responses used by the respective protocol version.
  Reexport the types from `v0_38::abci` as aliases for these names in the
  `abci` module, continuing the naming as used in older API.
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)).
- `[tendermint]` Rename `Signature::to_bytes` to `Signature::into_bytes`
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)).
- `[tendermint-abci]` Update the `Application` interface to CometBFT 0.38
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312))
- `[tendermint-rpc]`  Changes to support the RPC protocol in CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * Add `finalize_block_results` and `app_hash` fields to
    `endpoint::block_results::Response`.
  * The `deliver_tx` field is renamed to `tx_result` in
    `endpoint::broadcast::tx_commit::Response`.
  * The `tx_result` field type changed to `ExecTxResult` in
    `endpoint::tx::Response`.
  * The `event::EventData::NewBlock` variant is renamed to `LegacyNewBlock`.
    The new `NewBlock` variant only carries fields relevant since CometBFT 0.38.
  * Removed `event::DialectEvent`, replaced with non-generic serialization
    helpers in `event::{v0_34, v0_37, v0_38}`. The `Deserialize` helpers in
    the latter two modules are aliased from common types that can support both
    fields added in CometBFT 0.38, `block_id` and `result_finalize_block`,
    as well as the fields present 0.37. Likewise for `DialectEventData`
    and other event data structure types.
  * Changed some of the serialization dialect helpers to only be
    used by the 0.34 dialect and remove generics. The current dialect's
    seralization is switched to the serde impls on the domain types in
    `tendermint`.
- `[tendermint]` Changes to support the RPC protocol in CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * Due to some attribute changes, the format emitted by `Serialize` is
    changed for `abci::response` types `CheckTx` and `FinalizeBlock`.
- [`tendermint-rpc`] Decode the earliest block data fields of the `sync_info`
  object in `/status` response and expose them in the `SyncInfo` struct:
  `earliest_block_hash`, `earliest_app_hash`, `earliest_block_height`,
  `earliest_block_time`
  ([\#1321](https://github.com/informalsystems/tendermint-rs/pull/1321)).
- `[tendermint-proto]` Align the return signature of the `encode_vec` and
  `encode_length_delimited_vec` methods in the `Protobuf` trait with
  `prost::Message` by directly returning `Vec<u8>`.
  ([\#1323](https://github.com/informalsystems/tendermint-rs/issues/1323))
  * Remove mandatory cloning in `Protobuf` methods and let callers decide on
    clone beforehand for original value access

### IMPROVEMENTS

- `[tendermint-proto]` Generate prost bindings for CometBFT 0.38
  under the `tendermint::v0_38` module
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312))
- `[tendermint]` Support for CometBFT 0.38:
  ([\#1312](https://github.com/informalsystems/tendermint-rs/pull/1312)):
  * Add conversions to and from `tendermint::v0_38` protobuf
    types generated in [`tendermint-proto`].
  * Add request and response enums under `v0_38::abci` to enumerate all requests
    and responses appropriate for CometBFT version 0.38.
  * Add request and response types under `abci` to represent the requests
    and responses new to ABCI++ 2.0 in CometBFT version 0.38. The names are
    `ExtendVote`, `FinalizeBlock`, `VerifyVoteExtension`.
- `[tendermint-rpc]` Support for CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * `Deserialize` implementations on `abci::Event`, `abci::EventAttribute`
    that correspond to the current RPC serialization.
  * Domain types under `abci::response` also get `Deserialize` implementations
    corresponding to the current RPC serialization.
  * `Serialize`, `Deserialize` implementations on `abci::types::ExecTxResult`
    corresponding to the current RPC serialization.
  * Added the `apphash_base64` serializer module.

## v0.32.2

Fixed a minor cargo metadata problem that gummed up the 0.32.1 release.

## v0.32.1

Fixed a bug with processing the `latest_block_result` endpoint result
in the RPC client set to the 0.34 compatibility mode.

### BUG FIXES

- `[tendermint-rpc]` Use compatibility mode in implementations
  of the `Client::latest_block_results` method
  ([\#1326](https://github.com/informalsystems/tendermint-rs/pull/1326))

## v0.32.0

*May 3rd, 2023*

This release notably comes with a fully featured [light client attack detector][attack-detector],
and introduces a [CLI for the light client][light-client-cli] for verifying headers,
detecting attacks against the light client, and reporting the evidence to primary and witness nodes.

It also adds a [`Verifier::verify_misbehaviour_header`][verifier-method] method for verifying
headers coming from a misbehaviour evidence.

Moreover, the [`Client`][client-trait] trait is now exposed by the `tendermint-rpc` without requiring
the `http-client` or the `websocket-client` feature flags to be enabled.

[light-client-cli]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-cli
[attack-detector]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client-detector
[verifier-method]: https://github.com/informalsystems/tendermint-rs/blob/6a4cd245b6f362832b974104b40be973dd0ef108/light-client-verifier/src/verifier.rs#L67
[client-trait]: https://github.com/informalsystems/tendermint-rs/blob/6a4cd245b6f362832b974104b40be973dd0ef108/rpc/src/client.rs#L49

### BREAKING CHANGES

- [`tendermint-light-client-verifier`] Rename `Verifier::verify`
  to `Verifier::verify_update_header` to better describe
  its purpose versus `Verifier::verify_misbehaviour_header`
  ([\#1294](https://github.com/informalsystems/tendermint-rs/issues/1294))

### FEATURES

- [`tendermint-light-client-detector`] Implement a light client
  attack detector, based on its Go version found in Comet
  ([\#1291](https://github.com/informalsystems/tendermint-rs/issues/1291))
- [`tendermint-light-client-verifier`] Add `Verifier::verify_misbehaviour_header`
  for verifying headers coming from a misbehaviour evidence.
  The verification for these headers is a bit more relaxed in order to catch FLA attacks.
  In particular the "header in the future" check for the header should be skipped.
  ([\#1294](https://github.com/informalsystems/tendermint-rs/issues/1294))

### IMPROVEMENTS

- [`tendermint-rpc`]: Export `Client` trait unconditionally, without
  having to specify either the `http-client` or `websocket-client`
  ([\#1235](https://github.com/informalsystems/tendermint-rs/issues/1235))
- [`tendermint`]: Loosen bounds of merkle hashing functions to accept borrowed data.
  ([\#1310](https://github.com/informalsystems/tendermint-rs/issues/1310))

## v0.31.1

*April 17th, 2023*

Expose the `TypedEvent` marker trait.

### FEATURES

- Expose the `tendermint::abci::event::TypedEvent
  ([\#1288](https://github.com/informalsystems/tendermint-rs/pull/1288))

## v0.31.0

*April 16th, 2023*

Upgrade signature crate versions and add a `TypedEvent` trait for ABCI events.

### BREAKING CHANGES

* `[tendermint, tendermint-p2p]` Bump `ed25519` to v2, `k256` to v0.13, and `signature` to v2

### IMPROVEMENTS

- [`tendermint`] Adds a new `TypedEvent` for encoding structured data in ABCI events
  ([\#1288](https://github.com/informalsystems/tendermint-rs/pull/1288)).
- [`tools/proto-compiler`] Parse and fetch proto dependencies as listed in the
  `buf.lock` file
  ([\#1293](https://github.com/informalsystems/tendermint-rs/pull/1293)).

## v0.30.0

*March 7th, 2023*

This release introduces support for multiple versions of CometBFT protocols.
Consumers of tendermint-rs crates, with the exception of `tendermint-abci`,
should be able to interoperate with CometBFT nodes based on 0.34.x and
0.37.x releases, or a combination of these.

### BREAKING CHANGES

- [`tendermint`] Version-specific definitions for ABCI `Request` and `Response`
  enums under `v0_34::abci` and `v0_37::abci`, containing only the method variants
  present in each of the respective protocol versions.
  `Request` and `Response` defined under `v0_37` are re-exported under
  the non-versioned `abci` module name, but the `SetOption` variant is not present
  in these latest versions of the enums.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-abci`] Change the frame length encoding in the ABCI wire protocol
  to unsigned varint, to correspond to the changes in Tendermint Core 0.37.
  No compatibility with 0.34 is provided at the moment.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-rpc`] Changed the signature of `WebSocketClient::new_with_config`
  to accept a `WebSocketConfig` struct value rather than an `Option`.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-proto`] The `serializers::evidence` module has been made private.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-rpc`] Bump `async-tungstenite` dependency version to 0.20,
  re-exporting `WebSocketConfig` from `tungstenite` 0.18
  ([\#1276](https://github.com/informalsystems/tendermint-rs/pull/1276)).

### IMPROVEMENTS

- [`tendermint-proto`] Generate prost bindings for Tendermint 0.34 and 0.37 side by side.
  The version-specific structs are placed under the `tendermint::v0_34` and 
  `tendermint::v0_37` module namespaces, respectively. The names under
  `tendermint::v0_37` are also re-exported under `tendermint`.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint`] New and updated ABCI domain types for Tendermint Core v0.37
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
- [`tendermint`] Protobuf conversions provided for both `v0_34` and `v0_37`
  versions of the generated [`tendermint-proto`] structs, where applicable.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
- [`tendermint-rpc`] Introduce `client::CompatMode`, enumerating protocol
  compatibility modes specifying the RPC data encoding used by the client.
  An `HttpClient` can be created with a selected mode specified in the new
  `builder` API, or have the mode changed afterwards (usually after
  version discovery) by the added `set_compat_mode` method.
  For `WebSocketClient`, the mode can only be specified at creation via the new
  `builder` API.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-abci`] Port ABCI application support to 0.37 Tendermint Core API.
  No legacy support for 0.34 is provided at the moment.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
- Derive `Hash` on `tendermint::Time`
  ([#1278](https://github.com/informalsystems/tendermint-rs/issues/1278))
- [`tendermint-light-client`] Show `max_clock_drift` in error raised when header
  is from the future
  ([\#1280](https://github.com/informalsystems/tendermint-rs/issues/1280))

## v0.29.1

*February 27th, 2023*

Improve debug output for Ed25519 keys.

### BUG FIXES

- `[tendermint]` Restore hex-formatting in debug output of Ed25519 keys.
  ([#1272](https://github.com/informalsystems/tendermint-rs/pull/1272))

## v0.29.0

*Feb 17th, 2023*

This release features modularity improvements for the cryptographic routines, as well as fixes related to block verification and the use of a consensus-friendly ed25519 crate.

### BREAKING CHANGES

- `[tendermint]` Make implementations of cryptographic primitives replaceable
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * Provide a `Sha256` trait in module `crypto` and make digest hashing
    implementations available through it.
  * Provide a `Verifier` trait in module `crypto::signature` to enable
    alternative implementations of signature verification available through it.
    An `Error` enum is defined in the same module, representing the error cases
    that can arise in the implementation in a deliberately opaque way.
  * The module `crypto::default` provides pure Rust implementations of the
    cryptographic traits. The module is made available by a
    new `rust-crypto` feature, enabled by default.
  * `merkle::simple_hash_from_byte_vectors` is made generic over an
    implementation of the new `MerkleHash` trait. Implementations for
    Rust-Crypto conformant digest objects and the non-incremental
    `crypto::Sha256` API are provided in the crate.
  * The `Header::hash` and `ValidatorSet::hash` methods are gated by the
    `rust-crypto` feature. Generic hashing methods not dependent on
    the default crypto implementations are added for both types,
    named `hash_with`.
  * Conversions to `account::Id` and `node::Id` from `PublicKey` and
    curve-specific key types are gated by the `rust-crypto` feature.
  * The `validator::Info::new` method is gated by the `rust-crypto` feature.
  * Remove a deprecated constant `signature::ED25519_SIGNATURE_SIZE`.

- `[tendermint-light-client-verifier]` Changes for the new Tendermint crypto API
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * The `rust-crypto` feature, enabled by default, guards the
    batteries-included implementation types: `ProdVerifier`, `ProdPredicates`,
    `ProdVotingPowerCalculator`.
  * Remove the `operations::hasher` API (`Hasher` and `ProdHasher`),
    made unnecessary by the new crypto abstractions in the `tendermint` crate.
  * The `VerificationPredicates` trait features a `Sha256` associated type
    to represent the hasher implementation, replacing the `&dyn Hasher`
    parameter passed to methods.
  * Change the type of the `VerificationErrorDetail::FaultySigner` field
    `validator_set` to `ValidatorSet`. This removes a hasher dependency from
    `CommitValidator`, and `ProdCommitValidator` is now an empty dummy type.

- `[tendermint-light-client]` Changes for the new Tendermint crypto API
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * The `rust-crypto` feature enables the default crypto implementations,
    and is required by the `rpc-client` and `unstable` features.
    `ProdForkDetector` is guarded by this feature, and is made a specific
    type alias to the hasher-generic `ProvidedForkDetector` type.
  * `LightClientBuilder` gets another type parameter for the Merkle hasher.
    Its generic constructors lose the `Hasher` parameter.

### BUG FIXES

- `[tendermint-light-client]` Fix verification of blocks between two trusted
  heights
  ([#1246](https://github.com/informalsystems/tendermint-rs/issues/1246))

### DEPENDENCIES

- `[tendermint, tendermint-p2p]` Replaced the `ed25519-dalek` dependency with
  `ed25519-consensus`
  ([#1046](https://github.com/informalsystems/tendermint-rs/pull/1046))

### IMPROVEMENTS

- Update all crates to the [2021 edition](https://doc.rust-
  lang.org/edition-guide/rust-2021/index.html) of the Rust language
  ([#1262](https://github.com/informalsystems/tendermint-rs/issues/1262))

## v0.28.0

*Dec 13, 2022*

This is primarily a security-related release, and although it's a breaking
release, the breaking changes are relatively minor.

It is highly recommended that all tendermint-rs light client users upgrade to
this version immediately.

### BREAKING

- `[tendermint-light-client-verifier]` Add `is_matching_chain_id`
  method to the `VerificationPredicates` trait
- `[tendermint-light-client-verifier]` Add a
  `chain_id` field to the `TrustedBlockState` struct

### IMPROVEMENTS

- `[tendermint-light-client-js]` Switch to serde-wasm-bindgen for marshalling
  JS values ([#1242](https://github.com/informalsystems/tendermint-rs/pull/1242))

### SECURITY

- `[tendermint-light-client]` Fix an issue where the light client was not
  checking that the chain ID of the trusted and untrusted headers match

## v0.27.0

*Nov 28, 2022*

Following on from the ABCI domain type-related work in v0.26.0, this release
deduplicates types across the `tendermint` and `tendermint-rpc` crates, and
makes better use of our domain types across the crates (a big thanks to
@mzabaluev here!).

@romac helped make the RPC query interface more ergonomic, and @hu55a1n1
implemented Rust equivalents for Tendermint Go's
[VerifyCommitLight](https://github.com/tendermint/tendermint/blob/a6dd0d270abc3c01f223eedee44d8b285ae273f6/types/validator_set.go#L722)
and
[VerifyCommitLightTrusting](https://github.com/tendermint/tendermint/blob/a6dd0d270abc3c01f223eedee44d8b285ae273f6/types/validator_set.go#L775)
methods for the light client.

Some additional convenience methods for the `Time` type were provided by
@scalalang2.

### BREAKING CHANGES

- `[tendermint]` Change hash fields' type from `Bytes`
  ([#1095](https://github.com/informalsystems/tendermint-rs/issues/1095)):

  | Struct                         | Field                 | Type      |
  | ------------------------------ | --------------------- | --------- |
  | `abci::request::OfferSnapshot` | `app_hash`            | `AppHash` |
  | `abci::response::Info`         | `last_block_app_hash` | `AppHash` |
  | `abci::response::InitChain`    | `app_hash`            | `AppHash` |
  | `Genesis`                      | `app_hash`            | `AppHash` |

- `[tendermint]` Remove method `AppHash::value`,
  replaced with non-allocating `AppHash::as_bytes`
  [#1232](https://github.com/informalsystems/tendermint-rs/pull/1232).
- `[tendermint-rpc]` Remove ABCI-related types, change the affected field types
  to standard Rust types or ABCI domain types in `[tendermint]`.
  ([#1090](https://github.com/informalsystems/tendermint-rs/issues/1090))
- `[tendermint-rpc]` Extract the `key` field from `query::Condition` and
  structure a `query::Condition` to have `key` and `operation` fields, since the
  `key` field is common to all conditions
  ([#1230](https://github.com/informalsystems/tendermint-rs/issues/1230))
- `[tendermint]` Rename `merkle::proof::Proof` to `ProofOps`
  ([#1234](https://github.com/informalsystems/tendermint-rs/pull/1234))
- `[tendermint-rpc]` Change the type of `/tx` response field `proof`
  to `tendermint::tx::Proof`
  ([#1233](https://github.com/informalsystems/tendermint-rs/issues/1233))

### IMPROVEMENTS

- `[tendermint]` Added `Time` methods `unix_timestamp` and `unix_timestamp_nanos`.
  ([#1175](https://github.com/informalsystems/tendermint-rs/issues/1175))
- `[light-client]` Added `validate`, `validate_against_trusted`, `verify_commit` and `verify_commit_against_trusted` methods to `PredicateVerifier`.
  ([#1222](https://github.com/informalsystems/tendermint-rs/issues/1222))
- `[tendermint-rpc]` Make `tendermint_rpc::Query`'s fields
  public and add a `Condition::key(&self) -> &str` method
  ([#1230](https://github.com/informalsystems/tendermint-rs/issues/1230))
- `[tendermint]` Add domain types `merkle::Proof` and `tx::Proof`,
  to represent protobuf messages `crypto.Proof` and `types.TxProof` respectively
  ([#1234](https://github.com/informalsystems/tendermint-rs/pull/1234))

## v0.26.0

*Oct 31, 2022*

The highlight of this release is the addition of domain types specifically for
ABCI. Previously, Rust-based Tendermint application developers would have had to
exclusively rely on the generated Protobuf types. Many thanks to @hdevalence for
the heavy lifting on this, and to @mzabaluev for the porting work after the
Tendermint v0.35 retraction!

While we will endeavour to keep this API as stable as possible, we know that we
will have to evolve it over the coming months to reduce duplication of
functionality and types across the ABCI module and RPC crate, so please expect
further breaking changes in subsequent breaking releases.

### BREAKING CHANGES

- `[tendermint]` Added domain types for ABCI
  ([#862](https://github.com/informalsystems/tendermint-rs/issues/862))
- `[tendermint-proto]` Use `Bytes` for byte array fields of ABCI protobuf types.
  ([#1203](https://github.com/informalsystems/tendermint-rs/pull/1203))

### BUG FIXES

- `[tendermint-rpc]` The encoding of the `hash` field for requests to the `/block_by_hash`
  endpoint has been changed to base64 (from hex) to accommodate discrepancies in
  how the Tendermint RPC encodes this field for different RPC interfaces
  ([#942](https://github.com/informalsystems/tendermint-rs/issues/942))
- Allow a `TrustThresholdFraction` of 1  
  ([#1208](https://github.com/informalsystems/tendermint-rs/issues/1208))

### ENHANCEMENTS

- `[tendermint-abci]` Deprecate `Client::set_option`.
  ([#1203](https://github.com/informalsystems/tendermint-rs/pull/1203))

### FEATURES

- `[tendermint-rpc]` Add support for the `/block_by_hash` RPC endpoint. See <https://docs.tendermint.com/master/rpc/#/Info/block_by_hash> for details ([#832](https://github.com/informalsystems/tendermint-rs/issues/832)).

## v0.25.0

*Sep 23, 2022*

This release follows from v0.23.9, with the v0.24 series skipped due to
Tendermint Core [abandoning the v0.35 and v0.36
releases](https://github.com/informalsystems/tendermint-rs/discussions/1179). As
such, it is a non-breaking change, and removes the need to pin one's
tendermint-rs dependencies to a specific version (as was the case for the v0.23
series).

This release still targets compatibility with Tendermint Core v0.34, and
specifically provides compatibility with v0.34.21.

### BUG FIXES

- `[tendermint-rpc]` Fix deserialization of `/block_results` response when it contains evidence for a duplicate vote
   ([#1194](https://github.com/informalsystems/tendermint-rs/issues/1194))

### DEPENDENCIES

- Unpin `time` dependency
  ([#1199](https://github.com/informalsystems/tendermint-rs/pull/1199))

### ENHANCEMENTS

- `[proto]` Do not generate types in `google::protobuf`
  ([#1188](https://github.com/informalsystems/tendermint-rs/issues/1188)).
- Add support for [Tendermint Core
  v0.34.21](https://github.com/tendermint/tendermint/blob/v0.34.21/CHANGELOG.md#v03421),
  which primarily involves a small addition to the configuration file
  ([#1198](https://github.com/informalsystems/tendermint-rs/pull/1198))

## v0.23.9

*Aug 5, 2022*

This minor release adds Basic authentication support for HTTP and WebSocket RPC
clients, in addition to some dependency updates.

We had to restrict our `time` dependency for some crates to a version range of
`>=0.3, <0.3.12` due to what seems to be a recent issue in `js-sys` causing our
no\_std support to break. We will undo this restriction as soon as the issue is
resolved.

### DEPENDENCIES

- `[rpc]` Update async-tungstenite dependency to 0.17
  ([#1165](https://github.com/informalsystems/tendermint-rs/issues/1165)).
- Update Prost to v0.11
  ([#1171](https://github.com/informalsystems/tendermint-rs/pull/1171))

### FEATURES

- `[tendermint-rpc]` Add support for HTTP Basic authentication to HTTP and WebSocket RPC clients
  ([#1169](https://github.com/informalsystems/tendermint-rs/issues/1169))

## v0.23.8

*Jul 22, 2022*

This release focuses on ensuring compatibility with Tendermint v0.34.20, which
introduces a [prioritized
mempool](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-067-mempool-refactor.md).
As per the release notes for `v0.23.8-pre.1`, this has a minor additive impact
on the ABCI and RPC interfaces in the fields that the `CheckTx` response
contains.

This release also contains some important dependency updates and minor bug
fixes.

### BUG FIXES

- `[tools/proto-compiler]` Annotate serde to fall back to `Default` for the
  omitted fields when deserializing `tendermint_proto::abci::ResponseInfo` struct,
  also providing deserialization for the response at the `/abci_info` RPC endpoint.
  ([#1132](https://github.com/informalsystems/tendermint-rs/issues/1132))

### DEPENDENCIES

- Update `k256` to v0.11 ([#1153](https://github.com/informalsystems/tendermint-rs/issues/1153))

### ENHANCEMENTS

- `[tendermint-proto,tendermint-rpc,tools]` Update to ensure compatibility with
  Tendermint v0.34.20 ([#1159](https://github.com/informalsystems/tendermint-rs/issues/1159))

## v0.23.8-pre.1

*Jun 29, 2022*

This pre-release targets Tendermint v0.34.20-rc0, which introduces a prioritized
mempool. This has a minor additive impact on the ABCI and RPC interfaces in the
fields that the `CheckTx` response contains.

Pre-releases will continue along this line until v0.34.20 is released.

### FEATURES

- `[tendermint-proto]` Regenerate protos from Tendermint
  v0.34.20-rc0, including prioritized mempool fields in `ResponseCheckTx`
  ([#1148](https://github.com/informalsystems/tendermint-rs/issues/1148))
- `[tendermint-rpc]` Update `broadcast_tx_*` result to include
  prioritized new mempool fields available from v0.34.20-rc0
  ([#1148](https://github.com/informalsystems/tendermint-rs/issues/1148))

## v0.23.7

*Apr 25, 2022*

A minor update to use the latest version of `prost`.

### DEPENDENCIES

- Update `prost` to v0.10 ([#1113](https://github.com/informalsystems/tendermint-
  rs/issues/1113))

## v0.23.6

*Mar 29, 2022*

A minor release that allows for a small UX improvement in the usage of the
`Client::genesis()` call in `tendermint-rpc`.

### DEPENDENCIES

- `[tendermint-light-client]` Upgrade
  [`contracts`](https://crates.io/crates/contracts) dependency to v0.6.2
  ([#1097](https://github.com/informalsystems/tendermint-rs/pull/1097))

### IMPROVEMENTS

- `[tendermint-rpc]` Allow users to specify the `AppState` type in the `Client::genesis()` function.
  ([#1106](https://github.com/informalsystems/tendermint-rs/issues/1106))

## v0.23.5

*Jan 13, 2022*

A single breaking change is provided by this release in order to move us closer
toward `no_std` support in both tendermint-rs and ibc-rs.

### BREAKING CHANGES

- `[tendermint-light-client]` Split out the verification functionality from the
  `tendermint-light-client` crate into its own `no_std`-compatible crate:
  `tendermint-light-client-verifier`. This helps move us closer to `no_std`
  compliance in both tendermint-rs and ibc-rs
  ([#1027](https://github.com/informalsystems/tendermint-rs/issues/1027))

## v0.23.4

*Jan 11, 2022*

This release exclusively focuses on removing `native-tls`/`openssl` from the
dependency tree and replacing it with `rustls`. This was previously incorrectly
configured in our `hyper-proxy` dependency.

### DEPENDENCIES

- `[tendermint-rpc]`: Switch `hyper-proxy` to use `rustls`, eliminating
  the only use of `native-tls` in tendermint-rs dependencies
  ([#1068](https://github.com/informalsystems/tendermint-rs/pull/1068))

## v0.23.3

*Dec 20, 2021*

Here we mainly attempt to provide a short-term workaround for
[\#1021](https://github.com/informalsystems/tendermint-rs/issues/1021) by
catering for both possible forms of JSON serialization for public keys.

### IMPROVEMENTS

- `[tendermint]` `Hash` is implemented for `tendermint::Time`
  ([#1054](https://github.com/informalsystems/tendermint-rs/pull/1054))

### WORKAROUNDS

- `[tendermint-rpc]` Allow deserialization of public keys from validator updates
  from `block_results` endpoint in multiple JSON formats until this is fixed in
  Tendermint
  ([#1021](https://github.com/informalsystems/tendermint-rs/issues/1021))

## v0.23.2

*Dec 7, 2021*

This release focuses on the removal of
[`chrono`](https://crates.io/crates/chrono) as our primary dependency for
dealing with time, and replaces it with the
[`time`](https://crates.io/crates/time) crate.

This is necessarily a breaking change, but is released as v0.23.2 as per our
current [versioning
scheme](https://github.com/informalsystems/tendermint-rs#versioning).

### BREAKING CHANGES

- `[tendermint]` Reform `tendermint::Time`
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030)):
  * The struct content is made private.
  * The range of acceptable values is restricted to years 1-9999
    (as reckoned in UTC).
  * Removed conversions from/to `chrono::DateTime<chrono::Utc>`.
  * Changes in error variants: removed `TimestampOverflow`, replaced with
    `TimestampNanosOutOfRange`; removed `ChronoParse`, replaced with `TimeParse`.
- `[tendermint-rpc]` Use `OffsetDateTime` and `Date` types provided by the `time` crate
  in query operands instead of their `chrono` counterparts.
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030))

### IMPROVEMENTS

- `[tendermint]` Deprecated `signature::ED25519_SIGNATURE_SIZE`
  in favor of `Ed25519Signature::BYTE_SIZE`
  ([#1023](https://github.com/informalsystems/tendermint-rs/issues/1023))
- Remove dependencies on the `chrono` crate.
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030))
- `[tendermint]` Improve `tendermint::Time`
  ([#1036](https://github.com/informalsystems/tendermint-rs/issues/1036),
   revised by [#1048](https://github.com/informalsystems/tendermint-rs/pull/1048)):
  * Restrict the validity range of `Time` to dates with years in the range
    1-9999, to match the specification of protobuf message `Timestamp`.
    Add an `ErrorDetail` variant `DateOutOfRange` to report when this
    restriction is not met.
  * Added a conversion to, and a fallible conversion from,
    `OffsetDateTime` of the `time` crate.

## v0.23.1

*Nov 15, 2021*

Minor bug fixes.

### BUG FIXES

- `[tools/proto-compiler]` Fixed our proto-compiler, which was producing
  protos that did not compile due to an incorrect Prost field annotation
  ([#1014](https://github.com/informalsystems/tendermint-rs/issues/1014))
- `[tendermint]` The `tendermint::node::Id` `Display` implementation now prints
  the hexadecimal string in lowercase
  ([#971](https://github.com/informalsystems/tendermint-rs/issues/971))

## v0.23.0

*Oct 27, 2021*

The main changes in this release involve upgrading to [Prost
v0.9](https://github.com/tokio-rs/prost/releases/tag/v0.9.0) and some
foundational changes to prepare for `no_std` support for some of our crates.

One of the main `no_std`-related changes in this release was to break out
configuration-related data structures from the `tendermint` crate into their own
crate (`tendermint-config`) as these structures depend on other crates which do
not yet support `no_std`.

### BREAKING CHANGES

- Upgraded Prost to the official v0.9 release to finally resolve the security
  issue introduced by v0.7
  ([#925](https://github.com/informalsystems/tendermint-rs/issues/925))
- `[tendermint, tendermint-config]` The `tendermint::config`
  module has now been broken out into its own crate (`tendermint-
  config`) to help towards facilitating `no_std` compatibility
  ([#983](https://github.com/informalsystems/tendermint-rs/issues/983))
- `[tendermint]` The `tendermint::node::info::OtherInfo::rpc_address`
  field type has been changed from `tendermint::net::Address`
  to `String` toward facilitating `no_std` compatibility
  ([#983](https://github.com/informalsystems/tendermint-rs/issues/983))
- `[tendermint]` The `tendermint::node::info::ListenAddress::to_net_address`
  method was replaced with a simple `as_str` method toward facilitating
  `no_std` compatibility ([#983](https://github.com/informalsystems/tendermint-
  rs/issues/983))

### FEATURES

- `[tendermint-rpc]` Add support for the `/block_search` RPC endpoint. See
  <https://docs.tendermint.com/v0.34.x/rpc/\#/Info/block_search> for details
  ([#832](https://github.com/informalsystems/tendermint-rs/issues/832))

## v0.22.0

*Sep 23, 2021*

This release targets numerous issues largely in support of
[ibc-rs](https://github.com/informalsystems/ibc-rs). The major breaking change
in this regard is in the
[API](https://github.com/informalsystems/tendermint-rs/blob/dd371372da58921efe1b48a4dd24a2597225df11/light-client/src/components/verifier.rs#L143)
we use to perform verification in the `tendermint-light-client` crate.

Toward `no_std` compatibility and flexibility in the way we handle error tracing
and reporting, we have also refactored the entire error handling system in
`tendermint-rs` to make use of
[flex-error](https://github.com/informalsystems/flex-error).

Finally, we are also (painfully) aware of the fact that our documentation does
not build for this release and apologize for this. We currently still depend on
Prost v0.7.0 and are awaiting a new release of Prost after v0.8.0 that does not
break our builds. We have
[\#978](https://github.com/informalsystems/tendermint-rs/pull/978) open in
preparation for this upgrade and will release a new version of `tendermint-rs`
as soon as a new Prost release is available.

See below for more specific detail as to what has changed in this release.

### BREAKING CHANGES

- All crates' error handling has been refactored to make use of
  [`flex-error`](https://github.com/informalsystems/flex-error/). This gives
  users greater flexibility in terms of the error handling/reporting systems
  they want to use and is a critical step towards `no_std` support.
  ([#923](https://github.com/informalsystems/tendermint-rs/pull/923))
- `[tendermint-rpc]` The `/tx` endpoint's `Request::hash` field has been changed
  from `String` to `tendermint::abci::transaction::Hash`
  ([#942](https://github.com/informalsystems/tendermint-rs/issues/942))
- `[tendermint-light-client]` The light client verification functionality has
  been refactored (including breaking changes to the API) such that it can be
  more easily used from both `tendermint_light_client` and `ibc-rs`
  ([#956](https://github.com/informalsystems/tendermint-rs/issues/956))
- `[tendermint-light-client]` Disable the `lightstore-sled` feature by default
  ([#976](https://github.com/informalsystems/tendermint-rs/issues/976))

### BUG FIXES

- `[tendermint-rpc]` The encoding of the `hash` field for requests to the `/tx`
  endpoint has been changed to base64 (from hex) to accommodate discrepancies in
  how the Tendermint RPC encodes this field for different RPC interfaces
  ([#942](https://github.com/informalsystems/tendermint-rs/issues/942))

### FEATURES

- `[tendermint-rpc]` Add support for the `/consensus_params` RPC endpoint. See
  <https://docs.tendermint.com/v0.34.x/rpc/\#/Info/consensus_params> for details
  ([#832](https://github.com/informalsystems/tendermint-rs/issues/832))
- `[tendermint-rpc]` Runtime query parsing (relevant to the `/subscribe` and
  `/tx_search` endpoints) has been reintroduced. This allows for client-side
  validation of queries prior to submitting them to a remote Tendermint node. An
  example of how to use this is available in the `tendermint-rpc` CLI (see [the
  README](https://github.com/informalsystems/tendermint-rs/tree/main/rpc#cli)
  for details).
  ([#859](https://github.com/informalsystems/tendermint-rs/issues/859))
- `[tendermint, tendermint-light-client]` Add support for Secp256k1 signatures
  ([#939](https://github.com/informalsystems/tendermint-rs/issues/939))

### IMPROVEMENTS

- `[tendermint-p2p]` The `SecretConnection` can now be split into two halves to
  facilitate full-duplex communication (must be facilitated by using each half
  in a separate thread).
  ([#938](https://github.com/informalsystems/tendermint-rs/pull/938))
- `[tendermint-light-client]` Model-based tests are now disabled by default and
  can be enabled through the `mbt` feature
  ([#968](https://github.com/informalsystems/tendermint-rs/issues/968))
- `[tendermint, tendermint-rpc]` Derive `Eq` on `SignedHeader` and `Commit`
  ([#969](https://github.com/informalsystems/tendermint-rs/issues/969))
- `[tendermint-light-client]` Add `WebSocketClient::new_with_config` to specify
  the WebSocket connection settings
  ([#974](https://github.com/informalsystems/tendermint-rs/issues/974))
- `[tendermint-p2p]` Amino support is now implemented using the upstream
  `prost` crate, eliminating a dependency on `prost-amino`
  ([#979](https://github.com/informalsystems/tendermint-rs/pull/979))

## v0.21.0

*Jul 20, 2021*

This release introduces several minor breaking changes (see below), among other
improvements, that clean up a few RPC-related data structures and ensure better
correctness of the `TrustThresholdFraction` data structure when constructing
and deserializing it.

A [security issue](https://github.com/informalsystems/tendermint-rs/issues/925)
was reported in `prost` v0.7, and we attempted to upgrade to v0.8, but we are
still awaiting one [bug fix](https://github.com/tokio-rs/prost/issues/502) in
v0.8 before we can upgrade. The moment that is fixed in `prost`, we will upgrade
to v0.8 and provide another `tendermint-rs` release.

### BREAKING CHANGES

- `[tendermint-rpc]` Remove the `TmEvent` and `Attribute` structs and replace
  them with their equivalent domain types from the `tendermint` crate
  ([#918](https://github.com/informalsystems/tendermint-rs/issues/918))
- `[tendermint]` The `TrustThresholdFraction` struct can now only be constructed
  by way of its `new` constructor. Deserialization also now makes use of this
  constructor, facilitating better validation. The `numerator` and `denominator`
  fields can be accessed (read-only) via their respective methods, since the
  fields are now private.
  ([#924](https://github.com/informalsystems/tendermint-rs/issues/924))

### BUG FIXES

- `[tendermint]` Update Genesis for Tendermint v.0.34.x ([#917](https://github.com/informalsystems/tendermint-rs/pull/917))
- `[tendermint-rpc]` Fix bug where `NewBlock` events emitted by Tendermint could not be parsed because of a missing field ([#930](https://github.com/informalsystems/tendermint-rs/issues/930))

### IMPROVEMENTS

- `[tendermint-proto]` Regenerate the Rust equivalents of the Tendermint
  Protobuf structures for Tendermint v0.34.9
  ([#871](https://github.com/informalsystems/tendermint-rs/issues/871))
- `[tendermint-rpc]` Add `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` trait
  bounds to the RPC URL types
  ([#919](https://github.com/informalsystems/tendermint-rs/issues/919))
- `[tendermint-rpc]` Propagate JSON-RPC errors through the Rust subscription ([#932](https://github.com/informalsystems/tendermint-rs/issues/932))

## v0.20.0

*Jun 22, 2021*

This release's number is bumped up to v0.20.0 due to two minor breaking changes
in our public APIs (see the breaking changes section below for details).

Also, since nobody was really making use of the Light Node, we decided to remove
its crate from the repo for now. If anyone needs it back, please contact us and
we'll restore it (although, we are considering migrating any and all binaries to
their own repositories in the future to separate library-level concerns from
operational ones).

The `tendermint-p2p` crate is still undergoing significant expansion (thanks to
@xla and @melekes). A lot's been done and we're in the process of finalizing
this new architecture, which will form the basis for future work towards
building more Tendermint nodes in Rust. More on this in future
releases.

Other than that, this release mainly contains various small bug fixes,
improvements and dependency updates.

### BREAKING CHANGES

* `[tendermint-p2p]` Remove superfluous module name suffixes in `p2p::error` ([#898](https://github.com/informalsystems/tendermint-rs/pull/898))
* `[tendermint]` Rename `time::Time::to_rfc3339` to `as_rfc3339` to be
  consistent with Rust's [self reference
  conventions](https://rust-lang.github.io/rust-clippy/master/index.html#wrong_self_convention)
  ([#910](https://github.com/informalsystems/tendermint-rs/pull/910))

### BUG FIXES

* `[tendermint-abci,tendermint-rpc]` Fix DeliverTx response deserialization
  issues with `gas_wanted` and `gas_used` fields
  ([#876](https://github.com/informalsystems/tendermint-rs/issues/876))
* `[tendermint]` Update TendermintConfig for Tendermint v.0.34.x ([#897](https://github.com/informalsystems/tendermint-rs/issues/897))
* `[tendermint]` Better handling of optional values in TendermintConfig ([#908](https://github.com/informalsystems/tendermint-rs/issues/908))

### IMPROVEMENTS

* `[tendermint-light-client]` Replaced `tempdir` dev dependency (deprecated)
  with `tempfile`
  ([#851](https://github.com/informalsystems/tendermint-rs/issues/851))
* Updated the changelog process to use
  [unclog](https://github.com/informalsystems/unclog) format and unblock the PR
  merge process
  ([#891](https://github.com/informalsystems/tendermint-rs/pull/891)).
* `[tendermint]` Changed `tendermint::public_key::Secp256k1` to be an alias
  of `k256::ecdsa::VerifyingKey`
  ([#900](https://github.com/informalsystems/tendermint-rs/pull/900))

### REMOVED

* `[tendermint-light-node]` We removed the `light-node` crate from the repo since
  nobody's currently really using it. If anyone needs please log an issue and
  we'll restore it. It will, of course, remain accessible in the
  [repo history](https://github.com/informalsystems/tendermint-rs/tree/f207ecc0a7c071a54d63f159794b16a216741b38)
  for now.
  ([#879](https://github.com/informalsystems/tendermint-rs/issues/879))

## v0.19.0

*Apr 7, 2021*

This release primarily aims to enhance RPC and Light Client functionality,
thereby improving [`ibc-rs`] and fixing an important bug affecting the Light
Client ([#831]).

The RPC now supports TLS 1.2+ connections (through the use of [`rustls`]),
allowing for secure HTTP and WebSocket connections, as well as HTTP/HTTPS
proxies. This implies that the Light Client now also supports these types of
connections.

We additionally introduce two new crates:

* `tendermint-abci` - A lightweight, minimal framework for building Tendermint
  [ABCI] applications in Rust.
* `tendermint-light-client-js` - Exposes the Light Client's `verify` method to
  JavaScript/WASM. This implies that, for now, you need to bring your own
  networking functionality to practically make use of the Light Client's
  verification mechanisms.

Various relatively minor breaking API changes were introduced, and are listed
below.

### BREAKING CHANGES

* `[tendermint]` The `tendermint::block::CommitSig` enum's members have been
  renamed to be consistent with Rust's naming conventions. For example,
  `BlockIDFlagAbsent` is now renamed to `BlockIdFlagAbsent` ([#839])
* `[tendermint-light-client]` The Light Client no longer uses
  `tendermint::net::Address` to refer to peers, and instead uses the
  `tendermint_rpc::Url` type ([#835])
* `[tendermint-rpc]` The `Client::validators` method now requires a `Paging`
  parameter. Previously, this wasn't possible and, if the network had more than
  30 validators (the default for the RPC endpoint), it only returned a subset
  of the validators ([#831])
* `[tendermint-rpc]` The `Client::validators` method now requires a `Paging`
  parameter. Previously, this wasn't possible and, if the network had more than
  30 validators (the default for the RPC endpoint), it only returned a subset
  of the validators ([#831])
* `[tendermint-rpc]` The `SubscriptionClient` trait now requires a `close`
  method, since it assumes that subscription clients will, in general, use
  long-running connections. This should not, however, break any downstream
  usage of the clients ([#820])
* `[tendermint-rpc]` The `HttpClient` and `WebSocketClient` constructors now
  take any input that can be converted to a `tendermint_rpc::Url`. This should
  hopefully have minimal impact on projects using the code, but it might
  require some minor code changes in some cases - see the crate docs for more
  details ([#820])
* `[tendermint-rpc]` The `event::EventData::GenericJSONEvent` member has been
  renamed to `event::EventData::GenericJsonEvent` ([#839])
* `[tendermint-testgen]` The `TMLightBlock` data structure has been renamed to
  `TmLightBlock` to be consistent with Rust's naming conventions ([#839])

### FEATURES

* `[tendermint-abci]` Release minimal framework for building ABCI applications
  in Rust ([#794])
* `[tendermint-light-client]` The Light Client now provides support for secure
  (HTTPS) connections to nodes ([#835])
* `[tendermint-light-client-js]` First release of the
  `tendermint-light-client-js` crate to provide access to Tendermint Light
  Client functionality from WASM. This only provides access to the `verify`
  method at present, exclusively provides access to block verification. This
  does not include network access or the Light Client's bisection algorithm
  ([#812])
* `[tendermint-rpc]` Support for secure connections (`https://` and `wss://`)
  has been added to the Tendermint RPC clients, as well as support for HTTP
  proxies for HTTP clients ([#820])
* `[tendermint-rpc]` A `tendermint-rpc` CLI has been added to simplify
  interaction with RPC endpoints from the command line ([#820])

### IMPROVEMENTS

* `[tendermint]` IPv6 support has been added for `net::Address` ([#5])
* `[tendermint-rpc]` Add `wait_until_healthy` utility method for RPC clients
  to poll the `/health` endpoint of a node until it either returns successfully
  or times out ([#855])

### BUG FIXES

* `[tendermint-light-client]` Due to the RPC client's `validators` method
  sometimes only returning a subset of validators (for networks larger than 30
  validators), validator set hash calculations were failing. Now we are at
  least obtaining a full validator set ([#831])
* `[tendermint-rpc]` Fix intermittent deserialization failures of the consensus
  state response ([#836])

[#5]: https://github.com/informalsystems/tendermint-rs/issues/5
[#794]: https://github.com/informalsystems/tendermint-rs/pull/794
[#812]: https://github.com/informalsystems/tendermint-rs/pull/812
[#820]: https://github.com/informalsystems/tendermint-rs/pull/820
[#831]: https://github.com/informalsystems/tendermint-rs/issues/831
[#835]: https://github.com/informalsystems/tendermint-rs/issues/835
[#836]: https://github.com/informalsystems/tendermint-rs/issues/836
[#839]: https://github.com/informalsystems/tendermint-rs/pull/839
[#855]: https://github.com/informalsystems/tendermint-rs/pull/855
[ABCI]: https://github.com/tendermint/tendermint/tree/main/spec/abci/
[`ibc-rs`]: https://github.com/informalsystems/ibc-rs
[`rustls`]: https://github.com/ctz/rustls

## v0.18.1

*Feb 10, 2021*

The main focus for this minor release is fixing the rendering of our
[`tendermint-light-client` crate documentation][light-client-docs].

### BUG FIXES

* `[tendermint-proto]` Fix panic in evidence serialization in the case where we
  receive an empty evidence Protobuf structure ([#782])
* `[tendermint-light-node]` Upgrade `jsonrpc` dependency to v17.0 to fix security
  vulnerability in `hyper` v0.12.35 ([#803])
* `[tendermint-light-client]` Fix rendering of documentation on docs.rs ([#806])

[#782]: https://github.com/informalsystems/tendermint-rs/issues/782
[#803]: https://github.com/informalsystems/tendermint-rs/issues/803
[#806]: https://github.com/informalsystems/tendermint-rs/issues/806
[light-client-docs]: https://docs.rs/crate/tendermint-light-client/

## v0.18.0

*Jan 29, 2021*

This release is breaking due to significant dependency updates (see below).
It also introduces experimental support for
[backward verification][lc-backward-verif] for the Light Client,
feature-guarded behind the `unstable` feature.

The Light Client's storage system and its API were also improved.

### BREAKING CHANGES:

* `[all]` Update all crates to use the latest version of the following dependencies: ([#764])
  - `tokio` (`1.0`)
  - `hyper` (`0.14`)
  - `prost` (`0.7`)
  - `bytes` (`1.0`)
  - `async-tungstenite` (`0.12`)

### FEATURES

* `[light-client]` Add basic support for backward verification, behind a `unstable` feature flag. ([#361])
  Note: This feature is currently unstable and should not be relied on by downstream dependencies.

### BUG FIXES

* `[light-client]` Fix potential block ordering problem with sled-based lightstore ([#769])
* `[light-client]` Improve the API of the light store. ([#428])
* `[light-client]` The `sled`-backed lightstore is now feature-guarded under
   the `lightstore-sled` feature, which is enabled by default for now. ([#428])

[lc-backward-verif]: https://github.com/tendermint/spec/blob/master/rust-spec/lightclient/verification/verification_002_draft.md#part-v---supporting-the-ibc-relayer
[#361]: https://github.com/informalsystems/tendermint-rs/issues/361
[#428]: https://github.com/informalsystems/tendermint-rs/issues/428
[#764]: https://github.com/informalsystems/tendermint-rs/issues/764
[#769]: https://github.com/informalsystems/tendermint-rs/issues/769

## v0.17.1

*Jan 11, 2021*

This release primarily focuses on fixing [#774], which is critical to the Light
Client's correct and reliable operation.

### IMPROVEMENTS:

* `[rpc, tools]` The RPC probe has been moved into the `tools` folder and can
  now be easily executed against a Tendermint node running the kvstore app by
  way of [cargo make]. `tendermint-rpc` test coverage has been expanded here
  too. ([#758])

[#758]: https://github.com/informalsystems/tendermint-rs/pull/758
[cargo make]: https://github.com/sagiegurari/cargo-make

### BUG FIXES:

* `[tendermint]` `Time` values were not always formatted properly,
  causing the light client to sometimes return malformed light blocks ([#774])

[#774]: https://github.com/informalsystems/tendermint-rs/issues/774

## v0.17.0

*Dec 17, 2020*

This release is a significant breaking upgrade from v0.16.0 that primarily
targets compatibility with
[Tendermint v0.34](https://github.com/tendermint/tendermint/blob/main/UPGRADING.md#v0340)
and the [Cosmos Stargate release](https://stargate.cosmos.network/).

To highlight some of the major changes over the course of 3 release candidates
and this release, we have:

* Provided Tendermint v0.34.0 compatibility.
* Supported the development of [ibc-rs](https://github.com/informalsystems/ibc-rs/).
* Improved our model-based testing to provide complex test cases for the
  [Light Client](https://github.com/informalsystems/tendermint-rs/tree/main/light-client#testing).
* Refactored our serialization infrastructure to remove all Amino types and
  ensure Protobuf compatibility (see the [proto crate](./proto)). This includes
  a lot of work towards clearly separating our domain types from their
  serialization types.
* Started work on our [P2P layer] towards the eventual goal of implementing a
  Tendermint full node.
* Started work towards offering a WASM-based Tendermint Light Client.
* Introduced a WebSocket-based RPC client for interacting with the
  [Tendermint RPC](https://docs.tendermint.com/v0.34.x/rpc/), including event
  subscription.

Please see the following detailed release notes, as well as the crate
documentation, for further details.

### BREAKING CHANGES:

- `[rpc]` The RPC client interface has been refactored. The
  `Client` struct is now `HttpClient` and is enabled with the `http-client`
  feature. It provides all RPC endpoints except the subscription related ones.
- `[rpc]` The EventListener was replaced with a new and improved
  WebSocketClient for more robust event subscriptions. It can be enabled with the
  `websocket-client` feature. Subscriptions are exposed using unbounded
  channels. ([#516])
- `[tendermint]` Removed all traces of Amino, including `amino_types` modules.
  All types are now "domain types" implementing the `Protobuf` trait for Protobuf-encoding using Prost.
  ([#504], [#535], [#536], [#585])
- `[tendermint]` Protocol breaking changes for compatibility with Tendermint
  Core v0.34 (and the Cosmos Stargate release) ([#305]):
  - Validators are now sorted by voting power (descending)
    and address (ascending). ([#506])
  - Remove PubKey field from DuplicateVoteEvidence ([#502])
  - Fix hash of empty Merkle tree to comply with RFC6962 ([#498])
  - All binary encoding is done via protobuf3 instead of amino
    ([#504], [#535], [#536], [#585])
  - Various updates to JSON encoding ([#505])
- `[tendermint]` Direct serialization capabilities have been removed from the
  domain types. ([#639])
- `[tendermint]` Work has started on making it compulsory to construct domain
  types by way of their constructors to ensure validity. ([#639])

### FEATURES:

- `[light-client]` Introduce builder API for light client initialization
  ([#583])
- `[rpc]` The subscription client interface provides a structured `Query`
  mechanism to help ensure compile-time validity of subscription queries. ([#584])
- `[rpc]` Support unsubscribing from events ([#516])
- `[spec]` TLA+ for the Tendermint consensus algorithm including proof
  forks can only be caused by +1/3 Byzantine validators
  committing equivocation or amnesia attacks. ([#496])
- `[spec]` English spec of light client attacks and evidence required to
  correctly handle them ([#526])
- `[tendermint]` Implement `fmt::UpperHex` for `Transaction` ([#613])
- `[tendermint/proto-compiler]` Protobuf structs generator now also accepts
  commit IDs from the Tendermint Go repository ([#660])
- `[testgen]` Various features and improvements to support model-based testing with
  the [Apalache model checker] ([#414])

### IMPROVEMENTS:

- [`light-client]` Start using model-based testing to test Light Client
  executions against traces emitted from the TLA+ model ([#414])
- `[light-client]` Only require Tokio when `rpc-client` feature is enabled ([#425])
- `[rpc]` A `WebSocketClient` is now provided to facilitate event
  subscription for a limited range of RPC events over a WebSocket connection.
  See the [Tendermint `/subscribe` endpoint's](https://docs.tendermint.com/v0.34.x/rpc/#/Websocket/subscribe)
  and the `tendermint-rpc` crate's docs for more details ([#516])
- `[rpc]` The subscription client interface provides a structured `Query`
  mechanism to help ensure compile-time validity of subscription queries.
  See the crate docs and [#584] for details.
- `[rpc]` The RPC request and response types' fields are now all publicly
  accessible ([#636]).
- `[rpc]` A new RPC probe (in the `rpc-probe` directory) has been added to
  facilitate quick, pre-scripted interactions with a Tendermint node (via its
  WebSocket endpoint). This aims to help improve testing and compatibility
  between Tendermint in Go and Rust. ([#653])
- `[rpc]` The `WebSocketClient` now adds support for all remaining RPC requests
  by way of implementing the `Client` trait ([#646])
- `[rpc]` Support for the `tx_search` RPC endpoint has been added ([#701])
- `[rpc]` Responses that include events now automatically have their tag
  key/value pairs decoded from base64, where previously tag key/value pairs
  were Base64-encoded ([#717])
- `[rpc]` Support for the `consensus_state` RPC endpoint has been added ([#719])
- `[tendermint]` Remove `total_voting_power` parameter from `validator::Set::new` ([#739])
- `[tendermint, rpc, light-client]` Crates now compile to WASM on the
  `wasm32-unknown-unknown` and `wasm32-wasi` targets ([#463])
- `[tendermint, light-client]` Specify the proposer in the validator set of fetched light blocks ([#705])
- `[tendermint-proto]` Upgrade protobuf definitions to Tendermint Go v0.34.0 ([#737])
- `[testgen]` Compute `last_block_id` hash when generating a `LightChain` ([#745])
- Dependency updates:
  - Update sled to 0.34 ([#490])
  - Update k256 to v0.7 ([#752])
  - Remove tai64 crate  ([#603])

### BUG FIXES:

- `[light-client]` Fix bug where a commit with only absent signatures would be
  deemed valid instead of invalid ([#650])
- `[light-client]` Revert a change introduced in [#652] that would enable DoS attacks,
  where full nodes could spam the light client with massive commits (eg. 10k validators).
- `[rpc]` Correctly handles control and keep-alive messages ([#516], [#590])
- `[rpc]` More robust handling of concurrency issues ([#311], [#313])

[ibc-rs]: https://github.com/informalsystems/ibc-rs/
[#305]: https://github.com/informalsystems/tendermint-rs/issues/305
[#311]: https://github.com/informalsystems/tendermint-rs/issues/311
[#313]: https://github.com/informalsystems/tendermint-rs/issues/313
[#414]: https://github.com/informalsystems/tendermint-rs/issues/414
[#498]: https://github.com/informalsystems/tendermint-rs/issues/498
[#463]: https://github.com/informalsystems/tendermint-rs/issues/463
[#490]: https://github.com/informalsystems/tendermint-rs/issues/490
[#496]: https://github.com/informalsystems/tendermint-rs/issues/496
[#502]: https://github.com/informalsystems/tendermint-rs/issues/502
[#504]: https://github.com/informalsystems/tendermint-rs/issues/504
[#505]: https://github.com/informalsystems/tendermint-rs/issues/505
[#506]: https://github.com/informalsystems/tendermint-rs/issues/506
[#516]: https://github.com/informalsystems/tendermint-rs/pull/516
[#524]: https://github.com/informalsystems/tendermint-rs/issues/524
[#526]: https://github.com/informalsystems/tendermint-rs/issues/526
[#535]: https://github.com/informalsystems/tendermint-rs/issues/535
[#536]: https://github.com/informalsystems/tendermint-rs/issues/536
[#547]: https://github.com/informalsystems/tendermint-rs/issues/547
[#578]: https://github.com/informalsystems/tendermint-rs/pull/578
[#583]: https://github.com/informalsystems/tendermint-rs/pull/583
[#584]: https://github.com/informalsystems/tendermint-rs/pull/584
[#585]: https://github.com/informalsystems/tendermint-rs/issues/585
[#590]: https://github.com/informalsystems/tendermint-rs/issues/590
[#603]: https://github.com/informalsystems/tendermint-rs/pull/603
[#613]: https://github.com/informalsystems/tendermint-rs/pull/613
[#636]: https://github.com/informalsystems/tendermint-rs/pull/636
[#639]: https://github.com/informalsystems/tendermint-rs/pull/639
[#650]: https://github.com/informalsystems/tendermint-rs/issues/650
[#652]: https://github.com/informalsystems/tendermint-rs/pulls/652
[#654]: https://github.com/informalsystems/tendermint-rs/issues/654
[#660]: https://github.com/informalsystems/tendermint-rs/issues/660
[#663]: https://github.com/informalsystems/tendermint-rs/issues/663
[#665]: https://github.com/informalsystems/tendermint-rs/issues/665
[#653]: https://github.com/informalsystems/tendermint-rs/pull/653
[#667]: https://github.com/informalsystems/tendermint-rs/issues/667
[#672]: https://github.com/informalsystems/tendermint-rs/pull/672
[#679]: https://github.com/informalsystems/tendermint-rs/issues/679
[#425]: https://github.com/informalsystems/tendermint-rs/issues/425
[#646]: https://github.com/informalsystems/tendermint-rs/pull/646
[#690]: https://github.com/informalsystems/tendermint-rs/issues/690
[#701]: https://github.com/informalsystems/tendermint-rs/pull/701
[#705]: https://github.com/informalsystems/tendermint-rs/issues/705
[#717]: https://github.com/informalsystems/tendermint-rs/issues/717
[#719]: https://github.com/informalsystems/tendermint-rs/pull/719
[#737]: https://github.com/informalsystems/tendermint-rs/pull/737
[#739]: https://github.com/informalsystems/tendermint-rs/issues/739
[#745]: https://github.com/informalsystems/tendermint-rs/issues/745
[#752]: https://github.com/informalsystems/tendermint-rs/pull/752
[P2P layer]: https://github.com/informalsystems/tendermint-rs/tree/main/p2p


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

[light-client-dir]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client
[testgen-dir]: https://github.com/informalsystems/tendermint-rs/tree/main/testgen

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

[ADR-006]: https://github.com/informalsystems/tendermint-rs/blob/main/docs/architecture/adr-006-light-client-refactor.md
[ADR-007]: https://github.com/informalsystems/tendermint-rs/blob/main/docs/architecture/adr-007-light-client-supervisor-ergonomics.md

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
[tendermint-rpc]: https://github.com/informalsystems/tendermint-rs/tree/main/rpc#tendermint-rpc
[lite]: https://github.com/informalsystems/tendermint-rs/tree/main/tendermint/src/lite
[light-client-dir]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client

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
[tendermint v0.32]: https://github.com/tendermint/tendermint/blob/main/CHANGELOG.md#v0320
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
[tendermint v0.31]: https://github.com/tendermint/tendermint/blob/main/CHANGELOG.md#v0310
[#228]: https://github.com/tendermint/kms/pull/228
[0.5.0]: https://github.com/tendermint/kms/pull/220
[tendermint v0.30]: https://github.com/tendermint/tendermint/blob/main/CHANGELOG.md#v0300
[#219]: https://github.com/tendermint/kms/pull/219
[#205]: https://github.com/tendermint/kms/pull/219
[#181]: https://github.com/tendermint/kms/pull/181
[tendermint v0.29]: https://github.com/tendermint/tendermint/blob/main/CHANGELOG.md#v0290
[tendermint v0.28]: https://github.com/tendermint/tendermint/blob/main/CHANGELOG.md#v0280
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

