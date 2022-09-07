# ADR 001: Repository Structure

## Context

This crate began its life in the
[tendermint/kms](http://github.com/tendermint/kms) 
as a library of components necessary to build the Tendermint KMS. 
Other features were added to support additional tooling, like a Tendermint config parser and RPC client. 

We would now like to develop it further to support lite clients and eventually
full nodes. It has thus been split into its own repo to evolve independently of
the KMS. Here we propose an initial plan for the evolution of the repo towards
these goals.

## Decision

At this stage, the repository must address three concerns:

1) Maintain existing tendermint-rs roughly as is for the KMS and rpc client
users
2) Establish a Lite Client
3) Establish a Reactor Framework and initial implementations of Tendermint
reactor components

For now, we can consider that each of these will proceed as their own crate.
`tendermint-rs` already exists as the repo namesake, while `lite` and `reactor`
crates can be created in the root.


### Maintaining Tendermint-RS

We should support the current tendermint-rs as necessary, and not hasten to make
any sweeping changes to the structure until we better understand consumers.
Any changes must be well co-ordinated with at least the KMS. Also, the
`secret_connection` code should be 
[moved back to the KMS](https://github.com/interchainio/tendermint-rs/pull/21#issuecomment-529061992)

We may ultimately want to consider further breaking it up into crates. At
present, that might consist of the following:

- crypto
- tendermint-rs (depends on crypto) 
- config (depends on tendermint-rs)
- rpc (depends on tendermint-rs)

See some prior discussion about this
[here](https://github.com/interchainio/tendermint-rs/issues/7).

### Lite Client

Most of the necessary data types already exist (pubkey/signature, blockid,
vote), with support for serialization/deserialization. The crux of the lite
client is verifying validator sets by computing their merkle root, and verifying
commits by checking validator signatures. We have recently completed first
passes at these verifications - they need to be further reviewed, better
structured, and better tested.

Ideally, as much of the lite client code as possible is independent of the
particulars of pubkey/signature/blockid/vote/etc. The lite client should be
written generically, with its own traits, and the existing types made to
implement them. 

We should follow the [lite client spec closely](https://github.com/tendermint/tendermint/blob/main/docs/spec/consensus/light-client.md), and we should work in parallel on a TLA+ implementation.

Note the spec assumes a function `signers(commit)`, which returns the validators
for a given commit. In practice, the validator set itself is not in the commit,
so this requires fetching the validator set from a full node, computing the merkle root, 
and comparing it against the ValidatorsHash in the header. We must also ensure 
the hash of the header is included in the BlockID. This kind of stuff can be
abstracted from the core lite client code (ie. `signers(commit)`), but needs to
be supported in the actual implementation.

Note the 
[structure of commits is about to change in
Tendermint](https://github.com/tendermint/tendermint/issues/1648),
a big breaking change that will make blocks much smaller by eliminating
redundant information from the included votes (ie. just including the
signatures). The generic lite client should be able to abstract over such a
change just fine, but we'd of course have to update how the commit types
implement the lite client traits.

We should try to surface the validity of data as much as possible in the type
system, so we can clarify the levels of validity of our data. For instance, the
difference between a random validator set that's just been loaded and a
validator set that has been verified against some commits should have different
types.

We should also get started with a `lite` binary that reads from the rpc and performs a lite client sync.
Most of the RPC client is fleshed out, but we'll have to add support for a few
more fields and write tests.
    
### Reactors

The primary goals of the new reactor design are [deterministic simulation](https://www.youtube.com/watch?v=4fFDFbi3toc)
and a tight mapping between code and formal specification.

Deterministic simulation should allow us to simulate networks with hundreds (thousands?!) 
of nodes in a single process, and to deterministically replay simulations, allowing complex
executions to be studied and debugged. The framework will require further
investigations of Rust's concurrency model and existing frameworks.

Tight mapping between code and formal specification will allow us to more easily reason
about the correctness of the code, and, with some R&D work, automatically generate tests 
from the formal specification. It believes this will be helped via ideas like session types,
where as much as possible about the system's state and transitions is expressed
in the type system.

Work on both of these goals can begin independently. On the one hand, we should
be exploring Rust frameworks for deterministic simulation, and on the other we
should be writing the core reactor state machines, eventually to be plugged into
the simulation framework.

## Status

Proposed

## Consequences

### Positive

- Minimal changes to existing tendermint-rs
- Parallel execution paths
- Separation of concerns

### Negative

### Neutral

