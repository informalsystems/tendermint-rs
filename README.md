# Tendermint KMS 🔐

[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]

[build-image]: https://circleci.com/gh/tendermint/kms.svg?style=shield
[build-link]: https://circleci.com/gh/tendermint/kms
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/tendermint/kms/blob/master/LICENSE

Key Management System for Cosmos Validators.

https://cosmos.network/

## About

This repository contains `tmkms`, a lightweight service intended to be deployed
alongside the `gaiad` service (ideally on separate physical hosts) which provides
the following:

- **High-availability** access to validator signing keys
- **Double-signing** prevention even in the event the validator process is compromised
- **Hardware security module** storage for validator keys which can survive host compromise

## Status

Early adopters interested in using YubiHSM2 devices for Cosmos Validator key
storage can install the (forthcoming) `tmkms` v0.0.1 "Preview Release" using
the instructions below.

NOTE: This release does not yet support validator signing. See the following
issues for the remaining blockers:

- [ ] [Cosmos address (Bech32) support](https://github.com/tendermint/kms/issues/65)
- [ ] [Update to new signing message format & integration test](https://github.com/tendermint/kms/pull/55)
- [ ] [Full integration tests: Tendermint <-> KMS](https://github.com/tendermint/kms/issues/44)

## Supported Platforms

`tmkms` should build on any [supported Rust platform] which is also supported
by [libusb]. Here are some of the available tier 1, 2, and 3 Rust platforms which
are also supported by **libusb**:

NOTE: `tmkms` is presently tested on Linux/x86_64. We don't otherwise guarantee
support for any of the platforms below, but they theoretically meet the necessary
prerequisites for support.

### Operating Systems

- Linux
- Windows
- macOS
- FreeBSD
- NetBSD
- OpenBSD

### CPU Architectures

Rust supports the following CPU architectures:

**Tier 1:**

a.k.a. "guaranteed to work"

- `i686` (32-bit Intel)
- `x86_64` ("AMD64", "x64") - recommended

**Tier 2:**

a.k.a. "may work". Note that hardware accelerated AES (used for communication
with YubiHSM2) is not supported on these platforms and they have not been
rigorously tested for side-channel attack resistance.

- `arm` (32-bit ARM)
- `aarch64` (64-bit ARM)
- `mips` (32-bit MIPS)
- `mips64` (64-bit MIPS)
- `powerpc` (32-bit PowerPC)
- `powerpc64` (64-bit PowerPC)
- `sparc64` (64-bit SPARC)

## Installation

You will need the following prerequisites:

- **Rust** (stable; 1.27+): https://rustup.rs/
- **C compiler**: e.g. gcc, clang
- **pkg-config**
- **libusb** (1.0+). Install instructions for common platforms:
  - Debian/Ubuntu: `apt install libusb-1.0-0-dev`
  - RedHat/CentOS: `yum install libusb1-devel`
  - macOS (Homebrew): `brew install libusb`

To install `tmkms`, do the following:

1. (x86_64 only) Configure `RUSTFLAGS` environment variable: `export RUSTFLAGS=-Ctarget-feature=+aes`
2. Run the following to install Tendermint KMS using Rust's `cargo` tool:

```
$ cargo install tmkms
```

3. Copy the example `tmkms.toml` file to a local directory (e.g. `~/.tmkms`):

https://github.com/tendermint/kms/blob/master/tmkms.toml.example

Edit it to match your desired configuration.

## YubiHSM2 Setup

YubiHSM2 devices from Yubico are the main HSM solution supported by
Tendermint KMS at this time (Ledger support forthcoming!)

The `tmkms yubihsm` subcommand provides YubiHSM2 setup, information, and
testing features:

- `tmkms yubihsm detect` - list all YubiHSM2 devices detected via USB
- `tmkms yubihsm keys` - manage keys on the device
  - `tmkms yubihsm keys generate <id>` - generate an Ed25519 signing key with the given ID number (e.g. 1)
  - `tmkms yubihsm keys list` - list all Ed25519 signing keys in the YubiHSM2
  - `tmkms yubihsm keys test <id>` - perform a signing test using the given key

## Development

The following are instructions for setting up a development environment.
They assume you've already followed steps 1 & 2 from the Installation
section above (i.e. installed rustup and the noted nightly Rust released).

- Install **rustfmt**: `rustup component add rustfmt-preview`
- Install **clippy**: `rustup component add clippy-preview`

Alternatively, you can build a Docker image from the [Dockerfile] in the top
level of the repository, which is what is used to run tests in CI.

Before opening a pull request, please run the checks below:

### Testing

Run the test suite with:

```
cargo test --all-features
```

### Format checking (rustfmt)

Make sure your code is well-formatted by running:

```
cargo fmt
```

### Lint (clippy)

Lint your code (i.e. check it for common issues) with:

```
cargo clippy
```

## License

Copyright © 2018 Tendermint

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[supported Rust platform]: https://forge.rust-lang.org/platform-support.html
[Dockerfile]: https://github.com/tendermint/kms/blob/master/Dockerfile
