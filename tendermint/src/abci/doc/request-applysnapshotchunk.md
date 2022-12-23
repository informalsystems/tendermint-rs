Applies a snapshot chunk.

The application can choose to refetch chunks and/or ban P2P peers as
appropriate. Tendermint will not do this unless instructed by the
application.

The application may want to verify each chunk, e.g., by attaching chunk
hashes in `Snapshot::metadata` and/or incrementally verifying contents
against `app_hash`.

When all chunks have been accepted, Tendermint will make an ABCI [`Info`]
request to verify that `last_block_app_hash` and `last_block_height` match
the expected values, and record the `app_version` in the node state. It then
switches to fast sync or consensus and joins the network.

If Tendermint is unable to retrieve the next chunk after some time (e.g.,
because no suitable peers are available), it will reject the snapshot and try
a different one via `OfferSnapshot`. The application should be prepared to
reset and accept it or abort as appropriate.

[ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#applysnapshotchunk)
