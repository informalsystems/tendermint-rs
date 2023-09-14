- `[tendermint]` Improve and validate deserialization of `validator::Set`
  ([\#1348](https://github.com/informalsystems/tendermint-rs/issues/1348)).
  The `total_voting_power` field no longer has to be present in the format
  processed by `Deserialize`. If it is present, it is validated against the
  sum of the `voting_power` values of the listed validators. The sum value
  is also checked against the protocol-defined maximum.
- `[tendermint-proto]` In the `Deserialize` impls derived for
  `v*::types::ValidatorSet`, the `total_voting_power` field value is retrieved
  when present.
