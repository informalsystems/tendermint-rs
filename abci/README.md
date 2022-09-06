## tendermint-abci

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Audit Status][audit-image]][audit-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]

[ABCI] framework for building low-level applications for Tendermint in Rust.

## Requirements

- The latest stable version of Rust

## API

At present, this crate only exposes a synchronous, blocking API based on Rust's
standard library's networking capabilities. `async` client/server support is
planned in future updates.

The primary trait to be implemented by an ABCI application is the
[`Application`] trait. One of the core ideas here is that an ABCI application
must be able to be cloned for use in different threads, since Tendermint opens
4 connections to the ABCI server. See the [spec][tendermint-abci-spec] for
details.

## Examples

See [`src/application`](./src/application/) for some example applications
written using this crate.

To run the key/value store example application, from the `tendermint-abci`
crate's directory:

```bash
# Set your logging level through RUST_LOG (e.g. RUST_LOG=info)
# Binds to 127.0.0.1:26658
RUST_LOG=debug cargo run --bin kvstore-rs --features binary,kvstore-app

# Reset and run your Tendermint node (binds RPC to 127.0.0.1:26657 by default)
tendermint unsafe_reset_all && tendermint start

# Submit a key/value pair (set "somekey" to "somevalue")
curl 'http://127.0.0.1:26657/broadcast_tx_async?tx="somekey=somevalue"'

#{
#  "jsonrpc": "2.0",
#  "id": -1,
#  "result": {
#    "code": 0,
#    "data": "",
#    "log": "",
#    "codespace": "",
#    "hash": "17ED61261A5357FEE7ACDE4FAB154882A346E479AC236CFB2F22A2E8870A9C3D"
#  }
#}

# Query for the value we just submitted ("736f6d656b6579" is the hex
# representation of "somekey")
curl 'http://127.0.0.1:26657/abci_query?data=0x736f6d656b6579'

#{
#  "jsonrpc": "2.0",
#  "id": -1,
#  "result": {
#    "response": {
#      "code": 0,
#      "log": "exists",
#      "info": "",
#      "index": "0",
#      "key": "c29tZWtleQ==",
#      "value": "c29tZXZhbHVl",
#      "proofOps": null,
#      "height": "189",
#      "codespace": ""
#    }
#  }
#}
```

## License

Copyright © 2021 Informal Systems

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

[crate-image]: https://img.shields.io/crates/v/tendermint-abci.svg
[crate-link]: https://crates.io/crates/tendermint-abci
[docs-image]: https://docs.rs/tendermint-abci/badge.svg
[docs-link]: https://docs.rs/tendermint-abci/
[build-image]: https://github.com/informalsystems/tendermint-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3ARust
[audit-image]: https://github.com/informalsystems/tendermint-rs/workflows/Audit-Check/badge.svg
[audit-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3AAudit-Check
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/informalsystems/tendermint-rs/blob/main/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg

[//]: # (general links)

[ABCI]: https://docs.tendermint.com/master/spec/abci/
[`Application`]: ./src/application.rs
[tendermint-abci-spec]: https://github.com/tendermint/spec/blob/master/spec/abci/abci.md
