*Sep 23, 2021*

This release targets numerous issues largely in support of
[ibc-rs](https://github.com/informalsystems/ibc-rs). The major breaking change
in this regard is the
[API](https://github.com/informalsystems/tendermint-rs/blob/dd371372da58921efe1b48a4dd24a2597225df11/light-client/src/components/verifier.rs#L143)
we use to perform verification in the `tendermint-light-client` crate.

Toward `no_std` compatibility and flexibility in the way we handle error tracing
and reporting, we have also refactored the entire error handling system in
`tendermint-rs` to make use of
[flex-error](https://github.com/informalsystems/flex-error).

Finally, we are also (painfully) aware of the fact that our documentation does
not build for this release and apologize for this. We currently still depend on
Prost v0.7.0 and are awaiting a new release of Prost after v0.8.0 that does not
break our builds. We have
[\#978](https://github.com/informalsystems/tendermint-rs/pull/978) open in
preparation for this upgrade and will release a new version of `tendermint-rs`
as soon as a new Prost release is available.

See below for more specific detail as to what has changed in this release.
