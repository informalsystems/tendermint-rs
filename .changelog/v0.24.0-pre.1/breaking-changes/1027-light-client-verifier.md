- `[tendermint-light-client]` Split out the verification functionality from the
  `tendermint-light-client` crate into its own `no_std`-compatible crate:
  `tendermint-light-client-verifier`. This helps move us closer to `no_std`
  compliance in both tendermint-rs and ibc-rs
  ([#1027](https://github.com/informalsystems/tendermint-rs/issues/1027))
