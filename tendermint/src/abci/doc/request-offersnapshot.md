Offers a list of snapshots to the application.

`OfferSnapshot` is called when bootstrapping a node using state sync. The
application may accept or reject snapshots as appropriate. Upon accepting,
Tendermint will retrieve and apply snapshot chunks via
[`ApplySnapshotChunk`]. The application may also choose to reject a snapshot
in the chunk response, in which case it should be prepared to accept further
`OfferSnapshot` calls.

Only `app_hash` can be trusted, as it has been verified by the light client.
Any other data can be spoofed by adversaries, so applications should employ
additional verification schemes to avoid denial-of-service attacks. The
verified `app_hash` is automatically checked against the restored application
at the end of snapshot restoration.

See also the `Snapshot` data type and the [ABCI state sync documentation][ssd].

[ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#offersnapshot)

[ssd]: https://docs.tendermint.com/master/spec/abci/apps.html#state-sync
