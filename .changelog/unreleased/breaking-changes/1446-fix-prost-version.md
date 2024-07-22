- `[tendermint]` Bump `prost` and `prost-types` to their latest versions in tendermint crate.
  This was missed in [#1444](https://github.com/informalsystems/tendermint-rs/pull/1444),
  which only updated the two dependencies in `tendermint-rpc`, leading to duplicate versions
  of both crates to be present in the dependency graph.
  ([#1446](https://github.com/informalsystems/tendermint-rs/pull/1446))
