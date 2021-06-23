*Jun 22, 2021*

This release's number is bumped up to v0.20.0 due to two minor breaking changes
in our public APIs (see the breaking changes section below for details).

Also, since nobody was really making use of the Light Node, we decided to remove
its crate from the repo for now. If anyone needs it back, please contact us and
we'll restore it (although, we are considering migrating any and all binaries to
their own repositories in the future to separate library-level concerns from
operational ones).

The `tendermint-p2p` crate is still undergoing significant expansion (thanks to
@xla and @melekes). A lot's been done and we're in the process of finalizing
this new architecture, which will form the basis for future work towards
building more Tendermint nodes in Rust. More on this in future
releases.

Other than that, this release mainly contains various small bug fixes,
improvements and dependency updates.
