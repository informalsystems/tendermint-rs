# CHANGELOG

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
[ABCI]: https://docs.tendermint.com/master/spec/abci/
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
[Tendermint v0.34](https://github.com/tendermint/tendermint/blob/master/UPGRADING.md#v0340)
and the [Cosmos Stargate release](https://stargate.cosmos.network/).

To highlight some of the major changes over the course of 3 release candidates
and this release, we have:

* Provided Tendermint v0.34.0 compatibility.
* Supported the development of [ibc-rs](https://github.com/informalsystems/ibc-rs/).
* Improved our model-based testing to provide complex test cases for the
  [Light Client](https://github.com/informalsystems/tendermint-rs/tree/master/light-client#testing).
* Refactored our serialization infrastructure to remove all Amino types and
  ensure Protobuf compatibility (see the [proto crate](./proto)). This includes
  a lot of work towards clearly separating our domain types from their
  serialization types.
* Started work on our [P2P layer] towards the eventual goal of implementing a
  Tendermint full node.
* Started work towards offering a WASM-based Tendermint Light Client.
* Introduced a WebSocket-based RPC client for interacting with the
  [Tendermint RPC](https://docs.tendermint.com/master/rpc/), including event
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
  See the [Tendermint `/subscribe` endpoint's](https://docs.tendermint.com/master/rpc/#/Websocket/subscribe)
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
[P2P layer]: https://github.com/informalsystems/tendermint-rs/tree/master/p2p


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
