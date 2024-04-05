- `[tendermint]` Change `EventAttribute`'s `key` and `value` fields from `String` to `Vec<u8>` for Tendermint v0.34, as enforced by the Protobuf schema for Tendermint v0.34.
  `tendermint::abci::EventAttribute` is now an enum, to account for version 0.34 and 0.37+, therefore the `key`, `value` and `index` fields now have to be retrieved through the `key_str()`/`key_bytes`, `value_str()`/`value_bytes()` and `index()` methods.
  ([\#1400](https://github.com/informalsystems/tendermint-rs/issues/1400)).
