- [`tendermint`] Version-specific definitions for ABCI `Request` and `Response`
  enums under `v0_34::abci` and `v0_37::abci`, containing only the method variants
  present in each of the respective protocol versions.
  `Request` and `Response` defined under `v0_37` are re-exported under
  the non-versioned `abci` module name, but the `SetOption` variant is not present
  in these latest versions of the enums.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-abci`] Change the frame length encoding in the ABCI wire protocol
  to unsigned varint, to correspond to the changes in Tendermint Core 0.37.
  No compatibility with 0.34 is provided at the moment.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-rpc`] Changed the signature of `WebSocketClient::new_with_config`
  to accept a `WebSocketConfig` struct value rather than an `Option`.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-proto`] The `serializers::evidence` module has been made private.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
