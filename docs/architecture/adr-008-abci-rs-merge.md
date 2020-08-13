# ADR 008: Merge `abci-rs` into `tendermint-rs`

## Changelog
* 2020-08-11: First draft

## Context

As mentioned [here](https://github.com/informalsystems/tendermint-rs/issues/388#issue-646627068), one high level goal
for this `tendermint-rs` is to merge an already existing crate for build ABCI servers. There are three main crates for
this purpose:

1. [`rust-abci`](https://github.com/tendermint/rust-abci)
1. [`abci-rs`](https://github.com/devashishdxt/abci-rs)
1. [`abci2`](https://github.com/nomic-io/abci2)

### History

There was a [proposal](https://github.com/tendermint/rust-abci/issues/61) for rearchitecting `rust-abci` where a lot
of different aspects about `rust-abci` were discussed (for example, current `Application` trait does not enforce `Send +
Sync`, also `abci-rs` is not easy to work with when rest of the application is using `async` Rust etc.). A decision was
made to explore other architectures for `rust-abci` which are also compatible with `async` Rust.

### Current state

Currently, there are two new crates which were created in order to explore other architectures for `rust-abci`,
`abci-rs` and `abci2`.

`abci2`'s architecture is very minimal and is akin to a wrapper around raw `TcpStream` with `protobuf` encoding/decoding
of ABCI types. All the burden of handing requests of different types in different ways is left to the developer.

Unlike `abci2`, `abci-rs`' architecture is similar to `rust-abci` with some additional features, for example, support
for [`async_trait`](https://docs.rs/async-trait)s, ABCI logic [sanity checks](https://github.com/tendermint/rust-abci/issues/49)
with proper test coverage, support for [unix sockets](https://github.com/tendermint/rust-abci/issues/30).

## Decision

Absorb `abci-rs` and adopt types in `tendermint-rs` repository (using `tendermint-proto` crate, etc.).

## Status

Proposed

## Consequences

### Positive

- `abci-rs` will be kept up to date with the `tendermint-rs` development and remain compatible with other related
  crates.
- `abci-rs` can share dependencies with other related crates (for example, simpler types from `proto`, etc.).
- Less burden on developer/crate user.

### Negative/Neutral

- Developer is forced to use `async` Rust.
- Less flexibility for the developer/crate user.

## References

Issues in `tendermint-rs`:

- [High Level Goals for the next Third](https://github.com/informalsystems/tendermint-rs/issues/388)

Issues in `rust-abci`:

- [Rearchitecting Rust ABCI](https://github.com/tendermint/rust-abci/issues/61)
- [ABCI logic "sanity checks"](https://github.com/tendermint/rust-abci/issues/49)
- [Add support for unix sockets](https://github.com/tendermint/rust-abci/issues/30)

Documentation for `abci-rs`:

- [docs.rs](https://docs.rs/abci-rs/0.10.0/abci/)
