Check whether a transaction should be included in the mempool.

`CheckTx` is not involved in processing blocks, only in deciding whether a
transaction should be included in the mempool. Every node runs `CheckTx`
before adding a transaction to its local mempool. The transaction may come
from an external user or another node. `CheckTx` need not execute the
transaction in full, but can instead perform lightweight or statateful
validation (e.g., checking signatures or account balances) instead of more
expensive checks (like running code in a virtual machine).

[ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#checktx)