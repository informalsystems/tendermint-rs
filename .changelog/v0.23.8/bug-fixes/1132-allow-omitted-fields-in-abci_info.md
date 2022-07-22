- `[tools/proto-compiler]` Annotate serde to fall back to `Default` for the
  omitted fields when deserializing `tendermint_proto::abci::ResponseInfo` struct,
  also providing deserialization for the response at the `/abci_info` RPC endpoint.
  ([#1132](https://github.com/informalsystems/tendermint-rs/issues/1132))
