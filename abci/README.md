# abci

[![Continuous Integration](https://github.com/devashishdxt/abci-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/devashishdxt/abci-rs/actions?query=workflow%3A%22Continuous+Integration%22)
[![Crates.io](https://img.shields.io/crates/v/abci-rs)](https://crates.io/crates/abci-rs)
[![Documentation](https://docs.rs/abci-rs/badge.svg)](https://docs.rs/abci-rs)
[![License](https://img.shields.io/crates/l/abci-rs)](https://github.com/devashishdxt/abci-rs/blob/master/LICENSE-MIT)

A Rust crate for creating ABCI applications.

## ABCI Overview

ABCI is the interface between Tendermint (a state-machine replication engine) and your application (the actual state
machine). It consists of a set of methods, where each method has a corresponding `Request` and `Response` message type.
Tendermint calls the ABCI methods on the ABCI application by sending the `Request` messages and receiving the `Response`
messages in return.

ABCI methods are split across 4 separate ABCI connections:

- `Consensus` Connection: `InitChain`, `BeginBlock`, `DeliverTx`, `EndBlock`, `Commit`
- `Mempool` Connection: `CheckTx`
- `Info` Connection: `Info`, `SetOption`, `Query`
- `Snapshot` Connection: `ListSnapshots`, `LoadSnapshotChunk`, `OfferSnapshot`, `ApplySnapshotChunk`

Additionally, there is a `Flush` method that is called on every connection, and an `Echo` method that is just for
debugging.

To know more about ABCI protocol specifications, go to official ABCI [documentation](https://tendermint.com/docs/spec/abci/).

## Usage

Add `abci` in your `Cargo.toml`'s `dependencies` section:

```toml
[dependencies]
abci = "0.10"
```

Each ABCI application has to implement three core traits corresponding to all three ABCI connections, `Consensus`,
`Mempool` and `Info`.

> Note: Implementations of these traits are expected to be `Send + Sync` and methods take immutable reference of `self`.
So, internal mutability must be handled using thread safe (`Arc`, `Mutex`, etc.) constructs.

After implementing all three above mentioned `trait`s, you can create a `Server` object and use `Server::run()` to start
ABCI application.

`Server::run()` is an `async` function and returns a `Future`. So, you'll need an executor to drive `Future` returned
from `Server::run()`. `async-std` and `tokio` are two popular options. In `counter` example, we use `tokio`'s executor.

To know more, go to `examples/` to see a sample ABCI application.

### Documentation

- [`master`](https://devashishdxt.github.io/abci-rs/abci/)
- [`release`](https://docs.rs/abci-rs/)

### Features

- `use-tokio`: Enables `tokio` backend for running ABCI TCP/UDS server
  - **Enabled** by default.
- `use-async-std`: Enables `async-std` backend for running ABCI TCP/UDS server
  - **Disabled** by default.

> Features `use-tokio` and `use-async-std` are mutually exclusive, i.e., only one of them can be enabled at a time.
Compilation will fail if either both of them are enabled or none of them are enabled.

## Minimum Supported Versions

- Tendermint: [`v0.33.6`](https://github.com/tendermint/tendermint/releases/tag/v0.33.6)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
