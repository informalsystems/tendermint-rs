- `[tendermint-p2p]` The `SecretConnection` can now be cloned such that it can
  be accessed from multiple threads simultaneously. The main use case for this
  is to separate reading into one thread, and writing into another.
  ([#938](https://github.com/informalsystems/tendermint-rs/pull/938))
