# Cosmos KMS 🔐

[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]

[build-image]: https://circleci.com/gh/tendermint/kms.svg?style=shield
[build-link]: https://circleci.com/gh/tendermint/kms
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/tendermint/kms/blob/master/LICENSE

Key Management System for Cosmos Validators.

https://cosmos.network/

## About

This repository contains `cosmos-kms`, a lightweight service intended to be deployed
alongside the `gaiad` service (ideally on separate physical hosts) which provides
the following:

* **High-availability** access to validator signing keys
* **Double-signing** prevention even in the event the validator process is compromised
* **Hardware security module** storage for validator keys which can survive host compromise

## Installation

NOTE: Cosmos KMS is not ready for general use.

No releases of Cosmos KMS are presently available. Eager early adopters can
build and install the latest code from `master` as follows:

1. Install rustup (the Rust installer): https://rustup.rs/
2. Install Rust nightly: `$ rustup install nightly-2018-06-02`
3. Run the following to install Cosmos KMS from `master`:

```
$ cargo +nightly-2018-06-02 install --git https://github.com/tendermint/kms
```

Cosmos KMS presently requires a nightly Rust release. It should be able to
support stable Rust after the 1.27 stable release.

## Configuration

The `kms.toml.example` file contains an example configuration for the KMS.
Copy it to your preferred location, rename it to `kms.toml`, and edit it
to match your preferred configuration.

## Development

The following are instructions for setting up a development environment.
They assume you've already followed steps 1 & 2 from the Installation
section above (i.e. installed rustup and the noted nightly Rust released).

1. Install **rustfmt**: `rustup component add rustfmt-preview --toolchain nightly-2018-06-02`
2. Install **clippy**: `cargo +nightly-2018-06-02 install clippy`
3. Clone the repo: `$ git clone https://github.com/tendermint/kms`

Before opening a pull request, please run the checks below:

### Testing

Run the test suite with:

```
cargo +nightly-2018-06-02 test --features=dalek-provider,yubihsm-mockhsm
```

### Format checking (rustfmt)

Make sure your code is well-formatted by running:

```
cargo +nightly-2018-06-02 fmt
```

### Lint (clippy)

Lint your code (i.e. check it for common issues) with:

```
cargo +nightly-2018-06-02 clippy
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