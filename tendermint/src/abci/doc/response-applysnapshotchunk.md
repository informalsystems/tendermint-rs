Returns the result of applying a snapshot chunk and associated data.

The application can choose to refetch chunks and/or ban P2P peers as
appropriate. Tendermint will not do this unless instructed by the
application.

[ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#applysnapshotchunk)