[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Audit Status][audit-image]][audit-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust 1.44+][rustc-image]

# Light-Client

Implementation of the [Light Client Verification Protocol][light-client-verification].

## Requirements

Tested with Rust 1.44+, may work on older versions.

## Documentation

See documentation on [crates.io][docs-link].

## Example

The code below demonstrates the main use case for the Tendermint Light Client: syncing to the latest block, verifying it, and performing fork detection.

Please refer to the [`light_client` example](https://github.com/informalsystems/tendermint-rs/blob/master/light-client/examples/light_client.rs) for fully working code.

```rust
let primary_instance: Instance = make_instance(primary, primary_addr, primary_path);
let witness_instance: Instance = make_instance(witness, witness_addr, witness_path);

let mut peer_addr = HashMap::new();
peer_addr.insert(primary, primary_addr);
peer_addr.insert(witness, witness_addr);

let peer_list = PeerList::builder()
    .primary(primary, primary_instance)
    .witness(witness, witness_instance)
    .build();

let mut supervisor = Supervisor::new(
    peer_list,
    ProdForkDetector::default(),
    ProdEvidenceReporter::new(peer_addr),
);

let mut handle = supervisor.handle();

// Spawn the supervisor in its own thread.
std::thread::spawn(|| supervisor.run());

loop {
    // Synchronously query the supervisor via a handle
    let block = handle.verify_to_highest();

    match block {
        Ok(light_block) => {
            println!("[info] synced to block {}", light_block.height());
        }
        Err(e) => {
            println!("[error] sync failed: {}", e);
        }
    });

    std::thread::sleep(Duration::from_millis(800));
}
```

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

[crate-image]: https://img.shields.io/crates/v/tendermint-light-client.svg
[crate-link]: https://crates.io/crates/tendermint-light-client
[docs-image]: https://docs.rs/tendermint-light-client/badge.svg
[docs-link]: https://docs.rs/tendermint-light-client/
[build-image]: https://github.com/informalsystems/tendermint-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3ARust
[audit-image]: https://github.com/informalsystems/tendermint-rs/workflows/Audit-Check/badge.svg
[audit-link]: https://github.com/informalsystems/tendermint-rs/actions?query=workflow%3AAudit-Check
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/interchainio/tendermint-rs/blob/master/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-1.44+-blue.svg

[//]: # (general links)

[quick start]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/quick-start.md
[Tendermint]: https://github.com/tendermint/tendermint
[light-client-verification]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification/verification.md
