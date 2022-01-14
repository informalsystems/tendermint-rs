*Jan 13, 2022*

This pre-release targets Tendermint Core v0.35 and introduces a number of
breaking changes from the v0.23 series of tendermint-rs. We provide a
pre-release here so people can start experimenting with and preparing for
Tendermint v0.35 compatibility, but a number of refinements need to be made
before we can produce a v0.24.0 release.

One of the major changes involves the introduction of [domain types for
ABCI](https://github.com/informalsystems/tendermint-rs/pull/1022) in preparation
for the release of ABCI++ in Tendermint v0.36. It also includes a number of
fixes and backports from the v0.23.x series of tendermint-rs.
