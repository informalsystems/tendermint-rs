# Tendermint RPC Probe

The Tendermint RPC probe is an application that assists in testing the various
crates in this repository. It currently allows you to execute a quick probe of
a running [Tendermint] node, where a quick probe executes requests against all
of the [Tendermint RPC] endpoints (including subscriptions for different event
types), and saves all of the responses it gets as JSON files. These JSON files
can be used in testing in other crates.

[Tendermint]: https://github.com/tendermint/tendermint
[Tendermint RPC]: https://docs.tendermint.com/master/rpc/
