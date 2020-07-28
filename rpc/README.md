[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]

See the [repo root](/) for build status, license, rust version, etc.

# tendermint-rpc

A rust implementation of the core types returned by a Tendermint node's RPC 
endpoint. 
These can be used to deserialize JSONRPC responses.
All networking related features will be feature guarded to keep the dependencies small 
in cases where only the core types are needed.

## Documentation

See documentation on [crates.io][docs-link].

## The `client` Feature

Additionally, this crate includes an RPC client implementation to query Tendermint RPC endpoints.
To keep dependencies small when only the core types are needed, it has to be explicitly enabled via the `client` feature. 

### Related

- RPC [core types] in golang
  
- RPC endpoints REST interface documentation:
https://docs.tendermint.com/master/rpc/ 

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/tendermint-rpc.svg
[crate-link]: https://crates.io/crates/tendermint-rpc
[docs-image]: https://docs.rs/tendermint-rpc/badge.svg
[docs-link]: https://docs.rs/tendermint-rpc/

[//]: # (general links)


[tendermint]: https://github.com/tendermint/tendermint
[core types]: https://github.com/tendermint/tendermint/blob/8b4a30fada85fccd8f0cb15009344f1cbd8de616/rpc/core/types/responses.go#L1
[tendermint.rs]: https://crates.io/crates/tendermint
