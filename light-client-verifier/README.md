[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]

See the [repo root] for build status, license, rust version, etc.

# Light Client Verifier

The verification component of the [Light Client]. This is extracted in order to
be able to make use of verification predicates without any of the I/O and
dependencies on the Rust standard library (i.e. to facilitate `no_std` support).

## Documentation

See documentation on [crates.io][docs-link].

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint-light-client-verifier.svg
[crate-link]: https://crates.io/crates/tendermint-light-client-verifier
[docs-image]: https://docs.rs/tendermint-light-client-verifier/badge.svg
[docs-link]: https://docs.rs/tendermint-light-client-verifier/

[//]: # (general links)

[repo root]: https://github.com/informalsystems/tendermint-rs
[quick start]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/quick-start.md
[Tendermint]: https://github.com/tendermint/tendermint
[Light Client]: https://github.com/informalsystems/tendermint-rs/tree/main/light-client
