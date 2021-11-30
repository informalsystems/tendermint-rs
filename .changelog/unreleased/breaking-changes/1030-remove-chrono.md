- `[tendermint]` Reform `tendermint::Time`
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030)):
  * The struct content is made private.
  * The range of acceptable values is restricted to years 1-9999
    (as reckoned in UTC).
  * Removed conversions from/to `chrono::DateTime<chrono::Utc>`.
  * Changes in error variants: removed `TimestampOverflow`, replaced with
    `TimestampNanosOutOfRange`; removed `ChronoParse`, replaced with `TimeParse`.
- `[rpc]` Use `OffsetDateTime` and `Date` types provided by the `time` crate
  in query operands instead of their `chrono` counterparts.
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030))
