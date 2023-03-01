- [`tendermint-proto`] Generate prost bindings for Tendermint 0.34 and 0.37 side by side.
  The version-specific structs are placed under the `tendermint::v0_34` and 
  `tendermint::v0_37` module namespaces, respectively. The names under
  `tendermint::v0_37` are also re-exported under `tendermint`.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint`] New and updated ABCI domain types for Tendermint Core v0.37
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
- [`tendermint`] Protobuf conversions provided for both `v0_34` and `v0_37`
  versions of the generated [`tendermint-proto`] structs, where applicable.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
- [`tendermint-rpc`] Introduce `client::CompatMode`, enumerating protocol
  compatibility modes specifying the RPC data encoding used by the client.
  An `HttpClient` can be created with a selected mode specified in the new
  `builder` API, or have the mode changed afterwards (usually after
  version discovery) by the added `set_compat_mode` method.
  For `WebSocketClient`, the mode can only be specified at creation via the new
  `builder` API.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-abci`] Port ABCI application support to 0.37 Tendermint Core API.
  No legacy support for 0.34 is provided at the moment.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
