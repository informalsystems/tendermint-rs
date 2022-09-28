Signals the beginning of a new block.

Called prior to any [`DeliverTx`]s. The `header` contains the height,
timestamp, and more -- it exactly matches the Tendermint block header.

[ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#beginblock)