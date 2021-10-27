*Oct 27, 2021*

The main changes in this release involve upgrading to [Prost
v0.9](https://github.com/tokio-rs/prost/releases/tag/v0.9.0) and some
foundational changes to prepare for `no_std` support for some of our crates.

One of the main `no_std`-related changes in this release was to break out
configuration-related data structures from the `tendermint` crate into their own
crate (`tendermint-config`) as these structures depend on other crates which do
not yet support `no_std`.
