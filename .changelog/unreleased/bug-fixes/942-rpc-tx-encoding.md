- `[tendermint-rpc]` The encoding of the `hash` field for requests to the `/tx`
  endpoint has been changed to base64 (from hex) to accommodate discrepancies in
  how the Tendermint RPC encodes this field for different RPC interfaces
  ([#942](https://github.com/informalsystems/tendermint-rs/issues/942))
