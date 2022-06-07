- `[rpc]` Change data accessor methods `abci::Data::value` and
  `abci::Log::value` to `Data::as_bytes` and `Log::as_str`,
  returning a byte array slice and a string slice respectively.
  ([#1140](https://github.com/informalsystems/tendermint-rs/pull/1140))
