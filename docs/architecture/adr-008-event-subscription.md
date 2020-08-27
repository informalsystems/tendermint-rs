# ADR 008: RPC Client Event Subscription Mechanism

## Changelog

* 2020-07-23: Initial draft

## Context

The [Tendermint Light Client](../../light-client/) is one of many important
applications that will make use of the [RPC client](../../rpc/) to query
Tendermint full nodes for information relating to the blockchain.

Tendermint servers (e.g. full nodes) provide an [event
subscription][tm-event-subs] RPC endpoint to allow clients to receive
notifications of specific events as they happen (currently via a WebSockets
connection). We need to expose this subscription mechanism through the RPC client.

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
2. An appropriate concurrency model for drivers of the transport layer that
   allows the transport layer to operate independently of consumers of
   subscriptions. This is so that consumers don't block transport layer
   activities and vice-versa.

## Decision

### Assumptions

* All blocking operations that deal with I/O must be `async`.
* We will not be ["de-asyncifying" the RPC][issue-318] and will rather, in a
  future ADR, propose a synchronous architecture as well should we need one.

### Proposed Entities and Relationships

The entities in the diagram below are described in the following subsections.

![](assets/rpc-client-erd.png)

### `Event`

In terms of the subscription interface, this is ultimately what the end user is
most interested in obtaining. The `Event` type's structure is dictated by the
Tendermint RPC:

```rust
pub struct Event {
    /// The query that produced the event.
    pub query: String,
    /// The data associated with the event (determines its `EventType`).
    pub data: EventData,
    /// Event type and attributes map.
    pub events: Option<HashMap<String, Vec<String>>>,
}

pub enum EventData {
    NewBlock {
        block: Option<Block>,
        result_begin_block: Option<BeginBlock>,
        result_end_block: Option<EndBlock>,
    },
    Tx {
        tx_result: TxResult,
    },
    // ...
}
```

### `Subscription`

A `Subscription` here is envisaged as an entity that implements the
[Stream][futures-stream] trait, allowing its owner to asynchronously iterate
through all of the relevant incoming events. Use of such a subscription should
be as simple as:

```rust
while let Some(result_event) = subscription.next().await {
   match result_event {
      Ok(event) => { /* handle event */ },
      Err(e) => { /* terminate subscription and report error */ },
   }
}
```

For efficient routing of events to `Subscription`s, each `Subscription` should
have some kind of unique identifier associated with it (a `SubscriptionId`).
Each `Subscription` relates only to a single [`Query`](#query). Therefore, its
publicly accessible fields may resemble the following:

```rust
pub struct Subscription {
    pub id: SubscriptionId,
    pub query: Query,
    // ... other fields to help facilitate inter-task comms ...
}
```

`Subscription`s are created by a client - described in the following sub-section.

### Client Model

Users of the Tendermint RPC library may or may not want access to subscription
functionality. Since such functionality comes with additional overhead in terms
of resource usage and asynchronous task management, it would be optimal to
provide two separate client traits: one that only implements non-subscription
functionality, and one that only implements subscription functionality (where
clients could either implement one or both traits).

The interfaces of the two types of clients are envisaged as follows.

#### `Client`

This type of client would allow for interaction with all RPC endpoints except
those pertaining to subscription management. In our current implementation, this
client would only interact via the HTTP RPC endpoints (the `HttpClient` in the
entity diagram above).

**Note**: All `async` traits are facilitated by the use of [async-trait].

```rust
pub type Result<R> = std::result::Result<R, Error>;

#[async_trait]
pub trait Client {
    /// `/abci_info`: get information about the ABCI application.
    async fn abci_info(&self) -> Result<abci_info::AbciInfo>;

    /// `/abci_query`: query the ABCI application
    async fn abci_query<V>(
        &self,
        path: Option<abci::Path>,
        data: V,
        height: Option<Height>,
        prove: bool,
    ) -> Result<abci_query::AbciQuery>
    where
        V: Into<Vec<u8>> + Send;

    /// ...

    /// Perform a general request against the RPC endpoint
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request;
}
```

#### `SubscriptionClient`

A `SubscriptionClient` would be one that only provides access to subscription
functionality. In our current implementation, this client would interact with a
WebSocket connection to provide subscription functionality (the
`WebSocketSubscriptionClient` in the entity diagram above).

```rust
#[async_trait]
pub trait SubscriptionClient {
    /// `/subscribe`: subscribe to receive events produced by the given query,
    /// but specify how many event results can be buffered in the resulting
    /// subscription.
    ///
    /// Specifying a `buf_size` of zero should indicate to the transport that an
    /// **unbounded** channel should be used.
    async fn subscribe_with_buf_size(
        &mut self,
        query: String,
        buf_size: usize,
    ) -> Result<Subscription>;

    /// `/subscribe`: subscribe to receive events produced by the given query.
    async fn subscribe(&mut self, query: String) -> Result<Subscription> {
        // Use an unbounded channel by default
        self.subscribe_with_buf_size(query, 0)
            .await
    }

    /// `/unsubscribe`: unsubscribe from receiving events for the given
    /// subscription.
    ///
    /// This terminates the given subscription and consumes it, since it is no
    /// longer usable.
    async fn unsubscribe(&mut self, subscription: Subscription) -> Result<()>;
}
```

### Client Implementations

We envisage 3 distinct client implementations at this point:

* `HttpClient`, which only implements [`Client`](#client) (over HTTP).
* `WebSocketSubscriptionClient`, which only implements
  [`SubscriptionClient`](#subscriptionclient) (over a WebSocket connection).
* `HttpWebSocketClient`, which implements both [`Client`](#client) (over HTTP)
  and [`SubscriptionClient`](#subscriptionclient) (over a WebSocket connection).

#### Handle-Driver Concurrency Model

Depending on the underlying transport, a client may need a **transport driver**
running in an asynchronous context. As in the example of a WebSocket connection,
the rate at which one interacts with the WebSocket connection may differ to the
rate at which one interacts with `Event`s being produced by a `Subscription`, so
they necessarily need to run concurrently in different contexts.

Implementation of such a transport driver is transport-specific. Short-lived
request/response interactions (such as HTTP) would not require such a transport
driver, whereas a WebSocket connection would.

In cases where a driver is necessary, the client implementation would have to
become a **handle** to the driver, facilitating communication with it across
asynchronous tasks.

### `SubscriptionRouter`

All possible [`SubscriptionClient`](#subscriptionclient) implementations would
need some form of subscription management and event/result routing. A
`SubscriptionRouter` is proposed whose interface facilitates:

1. Immediate subscribe/unsubscribe request fulfilment.
2. Two-stage subscribe/unsubscribe request management (where a
   subscription/unsubscribe request can first be created in a "pending" state,
   and then either confirmed or cancelled).
3. Routing of incoming events to specific subscribers.

The interface for such an entity could resemble the following:

```rust
pub struct SubscriptionRouter {
    // ...
}

impl SubscriptionRouter {
    // Publish the given event to all subscribers matching the query associated
    // with the event.
    pub async fn publish(&self, ev: Event) {
        // ...
    }

    // Add a subscription with the specified parameters. The `event_tx`
    // parameter provides a way to transmit events to a `Subscription`.
    //
    // Once added with this method, the subscription is instantly created
    // (without first pending).
    pub fn add(&mut self, id: SubscriptionId, query: Query, event_tx: EventTx) {
        // ...
    }

    // Similar to `add`, but first creates a pending subscription. The
    // `result_tx` parameter provides the `SubscriptionRouter` with a way to
    // communicate to an async task about the result of the subscription
    // (whether it was confirmed or cancelled).
    pub fn add_pending(
        &mut self,
        id: SubscriptionId,
        query: Query,
        event_tx: EventTx,
        result_tx: ResultTx,
    ) {
        // ...
    }

    // Confirm a pending subscription.
    pub fn confirm_add(&mut self, id: &SubscriptionId) -> Result<()> {
        // ...
    }

    // Cancel a pending subscription, returning `err` through the `result_tx`
    // handle provided when calling `add_pending`.
    pub fn cancel_add(&mut self, id: &SubscriptionId, err: Error) -> Result<()> {
        // ...
    }

    // Remove the given subscription from the router.
    pub fn remove(&mut self, subs: Subscription) {
        // ...
    }

    // Initiate a pending subscription removal from the router. The `result_tx`
    // handle will be used to return confirmation or cancellation of the pending
    // removal.
    pub fn remove_pending(&mut self, subs: Subscription, result_tx: ResultTx) {
        // ...
    }

    // Confirm the removal of the subscription with the given ID.
    pub fn confirm_remove(&mut self, id: &SubscriptionId) -> Result<()> {
        // ...
    }

    // Cancel the pending removal, returning the given error through the
    // `result_tx` interface provided when initially calling `remove_pending`.
    pub fn cancel_remove(&mut self, id: &SubscriptionId, err: Error) -> Result<()> {
        // ...
    }
}
```

### `Query`

It is proposed that, using a *builder pattern*, we implement a subscription
`Query` interface that implements the full [query PEG][query-peg] provided by
the Go implementation of the RPC client. This would allow for compile-time
validation of queries.

The desired interface for constructing a query would look as follows:

```rust
// tm.event='NewBlock'
let query = Query::from(EventType::NewBlock);

// tm.event='Tx' AND tx.hash='XYZ'
let query = Query::from(EventType::Tx).and_eq("tx.hash", "XYZ");

// tm.event='Tx' AND tx.height=5
let query = Query::from(EventType::Tx).and_eq("tx.height", 5);
```

This interface could be implemented along the following lines.

```rust
// Query would only have constructors that either specified an event type
// (corresponding to a `tm.event='eventtype'` query) or a condition. There must
// be no constructor that allows for construction of an empty query.
pub struct Query {
    // A query can only match zero or one type of event.
    event_type: Option<EventType>,
    // A query can contain zero or more conditions.
    conditions: Vec<Condition>,
}

impl From<EventType> for Query {
    fn from(event_type: EventType) -> Self {
        Self {
            event_type: Some(event_type),
            conditions: Vec::new(),
        }
    }
}

impl Query {
    // An example of a constructor for `Operation::Eq`.
    pub fn eq(key: &str, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::new(key, Operation::Eq(value.into()))],
        }
    }

    // ...

    // Allows for a simple builder pattern.
    pub fn and_eq(mut self, key: &str, value: impl Into<Operand>) -> Self {
        self.conditions.push(Condition::new(key, Operation::Eq(value.into())));
        self
    }

    // ...
}

// Derived from https://github.com/tendermint/tendermint/blob/master/types/events.go
pub enum EventType {
    NewBlock,
    NewBlockHeader,
    NewEvidence,
    Tx,
    ValidatorSetUpdates,
}

pub struct Condition {
    key: String,
    op: Operation,
}

pub enum Operation {
    Eq(Operand),
    Lt(Operand),
    Lte(Operand),
    Gt(Operand),
    Gte(Operand),
    Contains(Operand),
    Exists,
}

// According to https://docs.tendermint.com/master/rpc/#/Websocket/subscribe,
// an operand can be a string, number, date or time. We differentiate here
// between integer and floating point numbers.
//
// It would be most useful to implement `From` traits for each of the different
// operand types to the `Operand` enum, as this would improve ergonomics.
pub enum Operand {
    String(String),
    Integer(i64),
    Float(f64),
    Date(chrono::Date),
    DateTime(chrono::DateTime),
}
```

## Status

Proposed

## Consequences

### Positive

* Provides relatively intuitive developer ergonomics (`Subscription` iteration
  to produce `Event`s).
* Mocking client functionality is relatively easy, allowing for a greater
  variety of testing (including simulating transport-level failures).

### Negative

* Requires an additional concurrent, potentially long-running `async` task to be
  concerned about (partially mitigated by the [handle-driver concurrency
  model](#handle-driver-concurrency-model)).
* Requires some knowledge of the use of unbounded and bounded channels for
  inter-task communication - or at least an understanding of what these choices
  mean for clients and how buffer size selection could impact cases where
  bounded channels are used.

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
[tokio-sync]: https://docs.rs/tokio/*/tokio/sync/index.html
[async-trait]: https://docs.rs/async-trait/*/async_trait/index.html

