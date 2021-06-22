This release's number is bumped up to v0.20.0 due to two minor breaking changes
in our public APIs:

1. `tendermint-p2p` crate's error naming conventions to make them more idiomatic
   (see [#898](https://github.com/informalsystems/tendermint-rs/pull/898)).
2. The `Time::to_rfc3339` function in the `tendermint` crate was renamed to\
   `Time::as_rfc3339` in line with Rust's self conventions (see
   [#910](https://github.com/informalsystems/tendermint-rs/pull/910)).

Also, since nobody was really making use of the Light Node, we decided to remove
its crate from the repo for now. If anyone needs it back, please contact us and
we'll restore it (although, we are considering migrating any and all binaries to
their own repositories in future to separate framework-level concerns from
operational ones).

The `tendermint-p2p` crate is still undergoing significant expansion (thanks to
@xla and @melekes!). A lot's been done and we're in the process of finalizing
this new architecture, which will form the basis for future work towards
building a Tendermint full node/validator in Rust. More on this in future
releases.

Other than that, this release mainly contains various small bug fixes,
improvements and dependency updates.
