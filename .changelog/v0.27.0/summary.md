*Nov 28, 2022*

Following on from the ABCI domain type-related work in v0.26.0, this release
deduplicates types across the `tendermint` and `tendermint-rpc` crates, and
makes better use of our domain types across the crates (a big thanks to
@mzabaluev here!).

@romac helped make the RPC query interface more ergonomic, and @hu55a1n1
implemented Rust equivalents for Tendermint Go's
[VerifyCommitLight](https://github.com/tendermint/tendermint/blob/a6dd0d270abc3c01f223eedee44d8b285ae273f6/types/validator_set.go#L722)
and
[VerifyCommitLightTrusting](https://github.com/tendermint/tendermint/blob/a6dd0d270abc3c01f223eedee44d8b285ae273f6/types/validator_set.go#L775)
methods for the light client.

Some additional convenience methods for the `Time` type were provided by
@scalalang2.
