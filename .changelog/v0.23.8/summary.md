*Jul 22, 2022*

This release focuses on ensuring compatibility with Tendermint v0.34.20, which
introduces a [prioritized
mempool](https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-067-mempool-refactor.md).
As per the release notes for `v0.23.8-pre.1`, this has a minor additive impact
on the ABCI and RPC interfaces in the fields that the `CheckTx` response
contains.

This release also contains some important dependency updates and minor bug
fixes.
