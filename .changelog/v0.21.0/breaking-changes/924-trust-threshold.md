- `[tendermint]` The `TrustThresholdFraction` struct can now only be constructed
  by way of its `new` constructor. Deserialization also now makes use of this
  constructor, facilitating better validation. The `numerator` and `denominator`
  fields can be accessed (read-only) via their respective methods, since the
  fields are now private.
  ([#924](https://github.com/informalsystems/tendermint-rs/issues/924))
