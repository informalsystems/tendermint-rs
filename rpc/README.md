# tendermint-rpc

A rust implementation of the core types returned by a Tendermint node's RPC 
endpoint. 
These can be used to deserialize JSONRPC responses.
All networking related features will be feature guarded to keep the dependencies small 
in cases where only the core types are needed.

Releases of this crate should be compatible to the same [tendermint] version as
the [tendermint.rs] crate with the same version number.    

## The `client` Feature

Additionally, this crate includes an RPC client implementation to query Tendermint RPC endpoints.
To keep dependencies small when only the core types are needed, it has to be explicitly enabled via the `client` feature. 

### Related

- RPC [core types] in golang
  
- RPC endpoints REST interface documentation:
https://docs.tendermint.com/master/rpc/ 

[tendermint]: https://github.com/tendermint/tendermint
[core types]: https://github.com/tendermint/tendermint/blob/8b4a30fada85fccd8f0cb15009344f1cbd8de616/rpc/core/types/responses.go#L1
[tendermint.rs]: https://crates.io/crates/tendermint