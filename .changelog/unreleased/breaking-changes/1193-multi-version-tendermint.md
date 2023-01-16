- [`tendermint`] Version-specific definitions for ABCI `Request` and `Response`
  enums under `v0_34::abci` and `v0_37::abci`, containing only the method variants
  present in each of the respective protocol versions.
  `Request` and `Response` defined under `v0_37` are re-exported under
  the non-versioned `abci` module name, but the `SetOption` variant is not present
  in these latest versions of the enums.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
