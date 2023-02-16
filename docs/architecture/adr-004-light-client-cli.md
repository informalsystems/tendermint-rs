# ADR 004: Light Client CLI

## Changelog

- 2020-02-09: Update about Abscissa
- 2020-01-22: Some content copied from old ADR-002

## Status

WIP.

## Context

The high level context for the light client is described in
[ADR-002](adr-002-light-client-adr-index.md).

For reference, a schematic of the light node is below:

![Light Node Diagram](assets/light-node.png).

Here we focus on how the Light Node process itself is composed.
The light node process must consider the following features:

- command line UX and flags
- config file
- logging
- error handling
- state management
- exposing RPC servers

Ideally, it can support all of this with a minimum of dependencies.

We'd like to be able to start a light node process and have it sync to the
latest height and stay synced while it runs.

## Decision

### Abscissa

[Abscissa](https://github.com/iqlusioninc/abscissa) is a framework for building CLI
tools in Rust by Tony Arcieri of Iqlusion.
It's focus is on security and minimizing dependencies.
The full list of dependencies can be found [here](https://github.com/iqlusioninc/abscissa#dependencies).

For instance, while it includes functionality for command-line option parsing like that
provided by `structopt` + `clap`, it does so with far less dependencies.

[Users](https://github.com/iqlusioninc/abscissa#projects-using-abscissa)
of note include the [Tendermint KMS](https://github.com/tendermint/kms)
for validators and the new
[Zebra ZCash full node](https://github.com/ZcashFoundation/zebra).

See the [introductory blog
post](https://iqlusion.blog/introducing-abscissa-rust-application-framework)
for more details.

### Config

Config includes:

- trusting period
- initial list of full nodes
- method (sequential or skipping)
- trust level (if method==skipping)

The configuration contains an initial list of full nodes (peers).
For the sake of simplicity, one of the peers is selected as the "primary", while the
rest are considered "backups". Most of the data is downloaded from the primary,
and double checked against the backups.

The state is considered "expired" if the difference between the current time and
the time from the trusted header is greater than a configurable "trusting
period". If at any point the state is expired, the node should log an error and
exit - it's needs to be manually reset.


### Initialization

The node is initialized with a trusted header for some height and a validator set for the next height.

The node may be initialized by the user with only a height and header hash, and
proceed to request the full header and validator set from a full node. This
reduces the initialization burden on the user, and simplifies passing this
information into the process, but for the state to be properly initialized it
will need to get the correct header and validator set before starting the light
client syncing protocol.

### State

The light node will need to maintain state including the current height, the
last verified and trusted header, and the current set of trusted validators.
