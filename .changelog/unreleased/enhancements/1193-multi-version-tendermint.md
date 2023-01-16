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
- [`tendermint-rpc`] Provide separate `Client` traits for Tendermint Core 0.34
  and 0.37, placed under the `v0_34::client` and `v0_37::client` modules.
  The latest version is re-exported as `crate::Client`.
  The websocket and HTTP client implement both traits, it's up to the
  application which one to import for use.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193))
- [`tendermint-abci`] Port ABCI application support to 0.37 Tendermint Core API.
  No legacy support for 0.34 is provided at the moment.
  ([#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)).
