- `[tendermint-rpc]` Support for CometBFT 0.38
  ([\#1317](https://github.com/informalsystems/tendermint-rs/pull/1317)):
  * `Deserialize` implementations on `abci::Event`, `abci::EventAttribute`
    that correspond to the current RPC serialization.
  * Domain types under `abci::response` also get `Deserialize` implementations
    corresponding to the current RPC serialization.
  * `Serialize`, `Deserialize` implementations on `abci::types::ExecTxResult`
    corresponding to the current RPC serialization.
  * Added the `apphash_base64` serializer module.
