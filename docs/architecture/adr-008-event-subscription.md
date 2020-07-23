# ADR 008: RPC Client Event Subscription Mechanism

## Changelog
* 2020-07-23: Initial draft

## Context

The [Tendermint Light Client](../../light-client/) is one of many important
applications that will make use of the [RPC client](../../rpc/) to query
Tendermint full nodes for information relating to the blockchain.

Tendermint servers (e.g. full nodes) provide an [event
subscription][tm-event-subs] RPC endpoint to allow clients to receive
notifications of specific events as they happen. We need to expose this
subscription mechanism to the RPC client.

In order to achieve this, we need:

1. An ergonomic interface for the RPC client that allows developers to subscribe
   to events produced by specific queries. Specifically, this interface must:
   1. Offer **subscription** functionality, where:
      1. A single **subscription** takes place to the **events** produced by a
         **query** (the [PEG] for which, at the time of this writing, is located
         [here][query-peg]).
      2. Callers must be able to create multiple distinct subscriptions.
      3. A subscription must offer an interface to allow for iteration over
         incoming events only relevant to that subscription (i.e. it should
         produce an **event stream**).
      4. It must be possible to, from outside of the RPC client, receive events
         from multiple subscriptions' event streams concurrently without
         interfering with/blocking each other.
      5. It must be possible to, from outside of the RPC client, handle
         subscriptions' event streams without blocking other RPC operations.
   2. Offer the ability to **unsubscribe** from specific queries (i.e. to
      terminate specific subscriptions).
2. A pub/sub architecture (with an appropriate concurrency model) that
   facilitates the subscription interface.

## Decision

### Assumptions

* All blocking operations that deal with I/O must be `async`
* We will not be ["de-asyncifying" the RPC][issue-318] and will rather, in a
  future ADR, propose a synchronous architecture as well should we need one.

### Proposed Entities and Relationships

The entities in the diagram below are described in the following subsections.
The diagram also gives some indication as to the proposed concurrency model for
the architecture and how the entities relate to each other.

![](assets/rpc-client-subscription-rels.png)

### Subscription

A `Subscription` here is envisaged as an entity that implements the
[Stream][futures-stream] trait, allowing its owner to asynchronously iterate
through all of the **relevant** incoming events. Use of such a subscription
should be as simple as:

```rust
while let Some(result_event) = subscription.next().await {
   match result_event {
      Ok(event) => { /* handle event */ },
      Err(e) => { /* terminate subscription and report error */ },
   }
}
```

Since the underlying events may not be produced at the same cadence as that of
the `Subscription` owner's processing, it is proposed here to make use of an
appropriate type of buffered channel to realize this. The size of this buffer
must be configurable through the RPC Client.

### RPC Pub/Sub

The RPC Client must asynchronously spawn a component `PubSub` whose
responsibilities are:

1. Manage subscriptions (subscribe/unsubscribe operations from the RPC Client).
2. Receive and deserialize incoming `Event`s.
3. Route `Event`s to `Subscription`s whose `Query` matches the `Event`.
4. Manage transport-level concerns (i.e. WebSockets connection: keep-alive
   messages, graceful connection closure, etc.).

Items (3) and (4) could be encapsulated into their own sub-components to allow
for better separation of concerns and therefore easier testing. For example, (3)
could be encapsulated into a `PubSubRouter` component and (4) into a component
that only handles transport-level concerns.

This is an *internal* component whose interface only matters to the RPC Client.

### RPC Client Interface Extension

The following interface is proposed as an extension to the existing [`Client`
structure][client]:

```rust
impl Client {
    /// Initiates a subscription for events produced by the given query. More
    /// details as to the query syntax is available from the [Tendermint
    /// `/subscribe` RPC endpoint documentation][tm-rpc-subscribe].
    /// 
    /// [tm-rpc-subscribe]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
    pub async fn subscribe(&mut self, query: &Query) -> Result<Subscription, Error> {
        // ...
    }

    /// Terminates the given subscription and consumes it.
    pub async fn unsubscribe(&mut self, subs: Subscription) -> Result<(), Error> {
       // ...
    }
}
```

### Query

Due to the potential complexity of queries, the proposed `Query` entity should
functionally take its lead from the [Go-based `Query` interface][tm-go-query].
This type of interface allows for simple short-term implementation (e.g. just
querying by way of event type), with eventual full implementation of the [query
PEG][query-peg].

`Query` should implement `FromStr`, which should parse queries and emit the
appropriate parsing errors:

```rust
let query = Query::from_str("tm.event='NewBlock'")?;
```

Matching of queries to events should also be simple:

```rust
if query.matches(event) {
   // do something
}
```

## Status

Proposed

## Consequences

### Positive

* Provides relatively intuitive developer ergonomics (`Subscription` iteration
  to produce `Event`s).
* Can be easily tested if the WebSockets transport layer is clearly encapsulated
  from the perspective of the RPC Pub/Sub component.

### Negative

* Requires an additional concurrent, potentially long-running `async` task to be
  concerned about.
* Requires careful selection of `Subscription` buffer size to avoid lost events
  and terminated subscriptions.

### Neutral

None

## References

* [\#313](https://github.com/informalsystems/tendermint-rs/issues/313)
* [\#311](https://github.com/informalsystems/tendermint-rs/issues/311)
* [\#458][pr-458]
* [Tendermint RPC subscription endpoint][tm-event-subs]

[tm-event-subs]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
[client]: https://github.com/informalsystems/tendermint-rs/blob/06ed36eaf7a74c0357b86d1d7450a2fec52ed6a0/rpc/src/client.rs#L20
[query-peg]: https://github.com/tendermint/tendermint/blob/98c595312af02037843b8fe74f0ee0625665448e/libs/pubsub/query/query.peg
[tm-go-query]: https://github.com/tendermint/tendermint/blob/98c595312af02037843b8fe74f0ee0625665448e/libs/pubsub/pubsub.go#L64
[PEG]: https://en.wikipedia.org/wiki/Parsing_expression_grammar
[futures-stream]: https://docs.rs/futures/*/futures/stream/trait.Stream.html
[pr-458]: https://github.com/informalsystems/tendermint-rs/pull/458
[issue-318]: https://github.com/informalsystems/tendermint-rs/issues/318
