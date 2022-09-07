- `[tendermint-rpc]` Runtime query parsing (relevant to the `/subscribe` and
  `/tx_search` endpoints) has been reintroduced. This allows for client-side
  validation of queries prior to submitting them to a remote Tendermint node. An
  example of how to use this is available in the `tendermint-rpc` CLI (see [the
  README](https://github.com/informalsystems/tendermint-rs/tree/main/rpc#cli)
  for details).
  ([#859](https://github.com/informalsystems/tendermint-rs/issues/859))
