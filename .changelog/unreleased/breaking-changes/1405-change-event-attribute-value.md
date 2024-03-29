- `[tendermint]` Change `EventAttribute` `value` from `String` to `Vec<u8>` for
  TM34. `key`, `value` and `index` now have to be called through `key()`,
  `value_str()` and `index()` to support both `Vec<u8>` and `String`.
  ([\#1400](https://github.com/informalsystems/tendermint-rs/issues/1400)).
