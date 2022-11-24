- `[tendermint-rpc]` Extract the `key` field from `query::Condition` and
  structure a `query::Condition` to have `key` and `operation` fields, since the
  `key` field is common to all conditions
  ([#1230](https://github.com/informalsystems/tendermint-rs/issues/1230))
