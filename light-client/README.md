[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]

See the [repo root] for build status, license, rust version, etc.

# Light-Client

Implementation of the [Light Client Verification][light-client-verification]
and [Fork Detection][light-client-detection] protocols.

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

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint-light-client.svg
[crate-link]: https://crates.io/crates/tendermint-light-client
[docs-image]: https://docs.rs/tendermint-light-client/badge.svg
[docs-link]: https://docs.rs/tendermint-light-client/

[//]: # (general links)

[repo root]: https://github.com/informalsystems/tendermint-rs
[quick start]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/quick-start.md
[Tendermint]: https://github.com/tendermint/tendermint
[light-client-verification]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification/verification.md
[light-client-detection]: https://github.com/informalsystems/tendermint-rs/tree/master/docs/spec/lightclient/detection
