*May 30th, 2024*

This release restores the commit verification interfaces of `PredicateVerifier` from tendermint-rs `0.35.0` and lower, but retains the performance improvements made in version `0.36.0`.

This version also brings a few new features to the HTTP RPC client, notably a way to specify the User-Agent to send along HTTP requests, as well as a way to override the underlying `reqwest` client.

Additionally, this release fixes a couple of issues with the `serde`-based deserialization of the `FinalizeBlock` and `Event` types.
