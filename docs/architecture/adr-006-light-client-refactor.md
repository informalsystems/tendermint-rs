# ADR 006: Light Client Refactor

## Changelog
- 2020-03-15: Initial draft
- 2020-03-23: New decomposition
- 2020-05-18: Concurrency
- 2020-06-26: Handle Abstraction

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
Refactor](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-043-blockchain-riri-org.md). The key take always from that work are:

1. Separate concerns into independently verifiable (by humans as well as
   computers) components.
2. Deterministic functions wherever possible
3. Use events to index parameters of component function executions to
   maintain alignment between specs and implementations.

## Decision

The LightClient will be refactored to coordinate interactions with
multiple peers. Two abstractions will be introduced; Instance
representing API for interacting with a single Peer and Supervisor,
which provides an API for interacting with a set of peers known as a
PeerList. The PeerList will have a single peer as the Primary and
multiple Peers available as Secondaries.

Each LightClient Instance will be configured to perform the light client
protocol against a single peer. The Instance will be decomposed into
independently verifiable components for:

    * IO: Fetching LightBlocks from the peer
    * Verifier: Verifying a LightBlock based on a TrustedState
    * Scheduler: Schedule the next block based on the last TrustedState
    * Clock: Provide the current time

In addition, cryptographic operations are also decomposed into three "sub-components":

    * Hasher: For hashing headers
    * CommitValidator: For validating commit integrity
    * VotingPowerCalculator: For tallying the voting power of a validator set in a commit and verifying signatures

Each component and operation will be represented as a trait allowing mock replacements
to be used during integration testing.

The Supervisor will:

* Manage the lifecycle of Instances
* Coordinate the execution of the fork detection protocol
* Publishing evidence to peers

## Integration

The goal is to allow the light client to be accessible from multiple
components running in different threads. One component will be the RPC server which will allow
external parties to interact with an internal light client. Another
immediate need is for the relayer to be able to fetch the latest trusted
state verified by the light client in order to relay IBC messages.

What we need is a component facade which exposes an interface that
provides synchronous interactions and can be mocked out during testing.
 That interface should be free of
serialization concerns for the user. Under the hood, that facade should
be facilitating interactions boundaries allowing components to operate
freely and safely in distinct threads or runtime.

```rust
fn main() {
    let mut light_client = Supervisor::new();

    // Get a handle specifically for the relayer
    let relayer_light_client = light_client.handle();

    let relayer = Relayer::new(light_client);
    ...

    // Run (consume Supervisor)
    light_client.run();
}
```

For lack of better naming we call this abstraction "Handle". The
Supervisor outlined above is executed by running in it's own thread. But
before doing so, it spawns a `Handle`. The Handle is clonable and can
be given to any component who wants safe, serialized access to the
Supervisor.

The implementation is a simple abstraction which facilitates
communication over channels. Events send across a channel include a
callback. This callback abstraction allows the outer handle to expose a
synchronous operation event if the delegation is inherently
asynchronous. Delegating at this level opens the door to exposing
synchronous interaction to thread pools and allowing components to
handle their concurrency optimization internally without burdening the
component user.

```rust
pub struct Handle {
    sender: channel::Sender<Event>,
}
...
impl Handle {
    pub fn verify_to_target(&mut self, height: Height) -> Result<Header, &'static str> {
        let (sender, receiver) = channel::bounded::<Event>(1);
        let callback = Callback::new(move |result| {
            let event = match result {
                Ok(header) => {
                    Event::Verified(header)
                },
                Err(err) => {
                    Event::FailedVerification()
                }
            };
            sender.send(event).unwrap();
        });

        self.sender.send(Event::VerifyToTarget(height, callback)).unwrap();

        match receiver.recv().unwrap() {
            Event::Verified(header) => {
                return Ok(header);
            },
            Event::FailedVerification() => {
                return Err("too bar");
            },
            _ => {
                return Err("that was unexpected");
            }

        }
    }
}

...
// Supervisor
pub struct Supervisor {
    ...
    sender: channel::Sender<Event>,
    receiver: channel::Receiver<Event>,
}

impl Supervisor {
    ...
    pub fn handle(&mut self) -> Handle {
        let sender = self.sender.clone();

        return Handle::new(sender);
    }
    ...
    pub fn run(mut self) {
        thread::spawn(move || {
            loop {
                let event = self.receiver.recv().unwrap();
                match event {
                    Event::Terminate(sender) => {
                        println!("Terminating light client");
                        sender.send(()).unwrap();
                        return
                    },
                    Event::VerifyToTarget(height, callback) => {
                        let outcome = self.verify_to_target(height);
                        callback.call(outcome);
                    },
                    _ => {
                        // NoOp?
                    },
                }
            }
        });
    }
}

```
## Status

Implemented:

* [Verification Predicates](https://github.com/informalsystems/tendermint-rs/blob/e2335c4/light-client/src/predicates.rs)
* [Verifier Component](https://github.com/informalsystems/tendermint-rs/blob/e2335c4/light-client/src/components/verifier.rs)
* [IO Component](https://github.com/informalsystems/tendermint-rs/blob/e2335c4/light-client/src/components/io.rs)
* [Scheduler Component](https://github.com/informalsystems/tendermint-rs/blob/e2335c4/light-client/src/components/scheduler.rs)
* [Supervisor](https://github.com/informalsystems/tendermint-rs/blob/e2335c40b1c5e1f7d47ee28ae5f9cc679730b7a2/light-client/src/supervisor.rs)
* [Example](https://github.com/informalsystems/tendermint-rs/blob/e2335c40b1c5e1f7d47ee28ae5f9cc679730b7a2/light-client/examples/light_client.rs)

## Consequences

### Positive
- Better Isolation of Concerns
- Deterministic Testing of the composition of components.
- Clear and easy to specify/verify concurrency model
- Components control their concurrency and can be locally optimized for
  bottlenecks while still exposing synchronous interfaces.

### Negative

### Neutral

## References

* [Light Client Spec](https://github.com/tendermint/spec/tree/bucky/light-reorg/spec/consensus/light)
* [Blockchain Reactor Refactor](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-043-blockchain-riri-org.md)
* [Reactor Experiments](https://github.com/informalsystems/reactor-experiments)
