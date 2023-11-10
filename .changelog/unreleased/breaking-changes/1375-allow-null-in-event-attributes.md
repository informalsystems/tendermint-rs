- `[tendermint]` Allow null values in `key` and `value` fields of
  `EventAttribute` when deserializing. The serialization schema for the fields
  is changed to `Option<String>`
  ([\#1375](https://github.com/informalsystems/tendermint-rs/issues/1375)).
