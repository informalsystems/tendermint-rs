*Jul 20, 2021*

This release introduces several minor breaking changes (see below), among other
improvements, that clean up a few RPC-related data structures and ensure better
correctness of the `TrustThresholdFraction` data structure when constructing
and deserializing it.

A [security issue](https://github.com/informalsystems/tendermint-rs/issues/925)
was reported in `prost` v0.7, and we attempted to upgrade to v0.8, but we are
still awaiting one [bug fix](https://github.com/tokio-rs/prost/issues/502) in
v0.8 before we can upgrade. The moment that is fixed in `prost`, we will upgrade
to v0.8 and provide another `tendermint-rs` release.
