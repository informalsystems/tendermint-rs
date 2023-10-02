- `[tendermint-rpc]` Changed `ErrorDetail` variants
  ([\#1362](https://github.com/informalsystems/tendermint-rs/pull/1362)):
  * Removed the `Hyper` and `InvalidUri` variants.
  * The `Http` variant now has `Error` from `reqwest` as the source.
  * Added the `InvalidProxy` variant.
  * The `tungstenite` dependency exposed through its `Error` type in
    WebSocket-related variants has been updated to version 0.20.x.
- `[tendermint-rpc]` Removed a `TryFrom<HttpClientUrl>` conversion for
  `hyper::Uri` as hyper is no longer a direct dependency
  ([\#1362](https://github.com/informalsystems/tendermint-rs/pull/1362)).
