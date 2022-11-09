*Oct 31, 2022*

The highlight of this release is the addition of domain types specifically for
ABCI. Previously, Rust-based Tendermint application developers would have had to
exclusively rely on the generated Protobuf types. Many thanks to @hdevalence for
the heavy lifting on this, and to @mzabaluev for the porting work after the
Tendermint v0.35 retraction!

While we will endeavour to keep this API as stable as possible, we know that we
will have to evolve it over the coming months to reduce duplication of
functionality and types across the ABCI module and RPC crate, so please expect
further breaking changes in subsequent breaking releases.
