# ADR 006: Light Client Refactor

## Changelog
- 2020-03-15: Initial draft
- 2020-03-23: New decomposition
- 2020-05-18: Concurrency

## Context

The light client protocol provides a method for verifying application
state without the execution of all preceding transactions. The
protocol for the light client is described in
[English](https://github.com/tendermint/spec/tree/bucky/light-reorg/spec/consensus/light)
while the rust implementation is described in
[ADR-002](adr-002-light-client-adr-index.md). This ADR outlines the
next iteration of the light client implementation in rust in which
outlines the runtime and concurrency concerns.

The basis for this work comes from the learnings of the [Reactor
Experiments](https://github.com/informalsystems/reactor-experiments) as
well as the [Blockchain Reactor
Refactor](https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-043-blockchain-riri-org.md). The key take aways from that work are:

1. Separate concerns into independently verifiable (by humans as well as
   computers) components.
2. Deterministic functions wherever possible
3. Use events to index parameters of component function executions to
   maintain alignment between specs and implementations.

## Decision

The LightClient will be refactored into a LightClient Instance and
LightClient Manager. Multiple Instance of the LightClient will be
spawned within the same process and managed by a LightClient Manager.
The Manager will multiplex the output of the instances into an internal
event loop. That event loop will be responsible for:

* Maintaining active set of peers and instances
* Routing events instances to and from relayer
* Accumulating LightBlocks for fork detection
* Publishing evidence to peers

Each lightClient Instance will be configured to perform the light client
protocol against a single peer. The Instance will be decomposed into
IO independently verifiable components for:

    * IO: Fetching LightBlocks from the peer
    * Verifier: Verifying a LightBlock based on a TrustedState
    * Scheduler: Schedule the next block based on the last TrustedState

Each components provides synchronous functions which can be
mocked out for testing complex scenarios. The Instance stores LightBlocks
it's received from it's configured peer in either Verified or Unverified
state.

### Concurrency

Concurrency is handled at the instance level. The LightClient manager
spawns multiple instance which run in parallel. If an instance should
fail (due to verification error or fault detection), the Manager can
replace them.

### Communication
Right now:
    * The relayer drives "update to height"
        * This can be an event input which the manager routes to the
          instances
    * The relayer needs to know when the header is available?
        * Should it block or should it simply wait for some kind of
          event
    * Should the relayer be modeled as an event loop

## Status

Development underway: 
* [LightClientt Instance](https://github.com/informalsystems/tendermint-rs/pull/237)

## Consequences

### Positive
- Better Isolation of Concerns
- Deterministic Testing of the composition of components.
- Clear and easy to specify/verify concurrency model

### Negative

### Neutral

## References

* [Light Client Spec](https://github.com/tendermint/spec/tree/bucky/light-reorg/spec/consensus/light)
* [Blockchain Reactor Refactor](https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-043-blockchain-riri-org.md)
* [Reactor Experiments](https://github.com/informalsystems/reactor-experiments)
