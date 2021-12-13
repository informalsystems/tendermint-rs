*Dec 7, 2021*

This release focuses on the removal of
[`chrono`](https://crates.io/crates/chrono) as our primary dependency for
dealing with time, and replaces it with the
[`time`](https://crates.io/crates/time) crate.

This is necessarily a breaking change, but is released as v0.23.2 as per our
current [versioning
scheme](https://github.com/informalsystems/tendermint-rs#versioning).
