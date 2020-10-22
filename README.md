# tendermint.rs

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Audit Status][audit-image]][audit-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]

[Tendermint] in Rust with [TLA+ specifications](/docs/spec).

Tendermint is a high-performance blockchain consensus engine 
for Byzantine fault tolerant applications written in any programming language.

## Requirements

Tested against the latest stable version of Rust. May work with older versions.

Compatible with the v0.34 series of [Tendermint Core][Tendermint].

## Documentation

See each component for the relevant documentation.

Libraries:

- [tendermint](./tendermint) - Tendermint data structures and
  serialization
- [tendermint-rpc](./rpc) - Tendermint RPC client and
  response types
- [light-client](./light-client) - Tendermint light client library for verifying
  signed headers, tracking validator set changes, and detecting forks

Binaries:

- [light-node](./light-node) - Tendermint light node to synchronize with a
  blockchain using the light client 

## Releases

Release tags can be found on [Github](https://github.com/informalsystems/tendermint-rs/releases).

Crates are released on crates.io.

## Contributing

The Tendermint protocols are specified in English in the
[tendermint/spec repo](https://github.com/tendermint/spec).
Any protocol changes or clarifications should be contributed there.

This repo contains the TLA+ specifications and Rust implementations for
various components of Tendermint. See the [CONTRIBUTING.md][contributing] to start
contributing.

## Versioning

We follow [Semantic Versioning](https://semver.org/). However, as we are
pre-v1.0.0, we use the MINOR version to refer to breaking changes and the PATCH
version for features, improvements, and fixes.

Only the following crates are covered by SemVer guarantees:

- tendermint
- tendermint-rpc

Other crates may change arbitrarily with every release for now.

We use the same version for all crates and release them collectively.

## Resources

Software, Specs, and Documentation

- [Tendermint Datastructures Spec](https://github.com/tendermint/spec)
- [Tendermint in Go](https://github.com/tendermint/tendermint)
- [Docs for Tendermint in Go](http://docs.tendermint.com/)

Papers

- [The latest gossip on BFT consensus](https://arxiv.org/abs/1807.04938)
- [Ethan Buchman's Master's Thesis on Tendermint](https://atrium.lib.uoguelph.ca/xmlui/handle/10214/9769)

## License

Copyright © 2020 Informal Systems

Licensed under the Apache License, Version 2.0 (the "License");
you may not use the files in this repository except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint.svg
[crate-link]: https://crates.io/crates/tendermint
[docs-image]: https://docs.rs/tendermint/badge.svg
[docs-link]: https://docs.rs/tendermint/
[build-image]: https://github.com/informalsystems/tendermint-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3ARust
[audit-image]: https://github.com/informalsystems/tendermint-rs/workflows/Audit-Check/badge.svg
[audit-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3AAudit-Check
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/interchainio/tendermint-rs/blob/master/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg

[//]: # (general links)

[tendermint-docs-link]: https://docs.rs/tendermint/
[tendermint-rpc-docs-link]: https://docs.rs/tendermint-rpc/
[Tendermint]: https://github.com/tendermint/tendermint
[tendermint-light-client-docs-link]: https://docs.rs/tendermint-light-client/
[tendermint-light-node-docs-link]: https://docs.rs/tendermint-light-node/
[contributing]: ./CONTRIBUTING.md
