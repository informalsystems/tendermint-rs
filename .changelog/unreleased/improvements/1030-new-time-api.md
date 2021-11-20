- Remove dependencies on the `chrono` crate.
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030))
- `[tendermint]` Improve `tendermint::Time`
  ([#1030](https://github.com/informalsystems/tendermint-rs/issues/1030)):
  * Restrict the validity range of `Time` to dates with years in the range
    1-9999, to match the specification of protobuf message `Timestamp`.
    Add an `ErrorDetail` variant `DateOutOfRange` to report when this
    restriction is not met.
  * Added a conversion to, and a fallible conversion from,
    `OffsetDateTime` of the `time` crate.
  * Added `Time` methods `checked_add` and `checked_sub`.
