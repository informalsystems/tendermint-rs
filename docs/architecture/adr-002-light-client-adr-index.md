# ADR 002: Light Client ADR Index

## Changelog

- 2019-10-30: First draft.
- 2019-12-27: Add diagram and align with the code.
- 2020-01-22: Refactor into multiple ADRs

## Status

ADR Index.

## Context

This is the first of multiple ADRs about the Tendermint light client in Rust.
Light client specification and implementation is an ongoing work in progress, in 
[English][english-spec],
[TLA+][tla-spec], 
[Go][go-impl],
and [Rust][rust-impl]. Note/TODO these links will soon be stale and need
updating.

It is especially important to review the Motivation and Structure of the Light
Client as described in the English Specification before reading these ADRs for
the necessary context.

Here we are concerned primarily with the implementation of a light node in Rust.
The light node process makes RPC requests to full nodes for blockchain data, uses
the core verification library to verify this data, and updates its state accordingly. 
It also makes requests to other full nodes to detect and report on forks.

That said, as much as possible, we would like the implementation here to be
reusable for the IBC protocol, which supports communiction between blockchains. 
In this case, instead of making RPC requests, IBC-enabled blockchains receive the relevant data in transactions and
verify it using the same core verification library. Thus implementations should
abstract over the source of data as necessary.
For more information on IBC, see the 
[spec repo](https://github.com/cosmos/ics),
especially the specifications for
[client
semantics](https://github.com/cosmos/ics/tree/master/spec/ics-002-client-semantics)
and [handler
interface](https://github.com/cosmos/ics/tree/master/spec/ics-025-handler-interface)

A complete implementation of the light node includes many components, each of
which deserves their own ADR. This ADR thus serves as an index for those, and
may be updated over time as other components come into view or
existing components are iterated upon.

Components include (TODO: links):

- ADR-003 - Core Verification Library: Data types, traits, and public API for
  the core verification protocol, with support for use in light nodes and IBC
  implemented
- ADR-004 - Command Line Interface: Choice of framework for the CLI and
  daemon, including config, arguments, logging, errors, etc.
- ADR-005 - Fork Detection Module: Simple module to manage an address book of 
  peers and search for conflicting commits


A schematic diagram of the light node, as taken from the [English
specification][english-spec], is provided below:

![Light Node Diagram](assets/light-node.png).


[english-spec]: https://github.com/tendermint/spec/tree/bucky/light-reorg/spec/consensus/light
[tla-spec]: https://github.com/interchainio/verification/tree/igor/lite/spec/light-client
[go-impl]: https://github.com/tendermint/tendermint/tree/main/lite2
[rust-impl]: https://github.com/interchainio/tendermint-rs/tree/master/tendermint-lite
