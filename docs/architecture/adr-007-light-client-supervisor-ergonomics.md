# ADR 007: Light Client Supervisor Ergonomics

## Changelog

* 2020-06-30: Initial draft

## Context

The initial approach to use callbacks in order to capture async responses
from requests issued by the `Handle` to its `Supervisor` introduces a couple of
has some drawbacks, i.e. try semantics, scope and borrow complexities. As the
general pattern seems to be that a bounded channel is constructed to transport
the response it could be reworked to be the actual value send with the event to
the Supervisor. Which in turn send back the response on the typed channel to the
Handle. As an added side-effect this could remove the need for the Callback
abstraction all together and fully embraces CSP style concurrency.

Furthermore we have async versions of most `Handle` methods, but no real
use-case driving it. If we find such needs, we should look into an interface
embracing `async/await`.

## Decision

Remove the callback abstraction in favour of passing channels in the events
exchanged directly as the pattern is present already in the current concrete
implementation of the `supervisor::Handle` anyway.

Remove async versions of the public methods until use-case drive their
implementation.

## Status

Proposed

## Consequences

### Positive

* Smaller public surface of the Handle abstraction
* Removal of Callback type
* Easier to reason code

### Negative

### Neutral

* No explicit async version of Supervisor methods

## References

* [Feedback on preceeding
  ADR](https://github.com/informalsystems/tendermint-rs/pull/185#pullrequestreview-439830876)
* [Tracking issue](https://github.com/informalsystems/tendermint-rs/issues/398)
