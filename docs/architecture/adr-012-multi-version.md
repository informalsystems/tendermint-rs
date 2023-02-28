# ADR 012: Support for multiple versions of CometBFT Core

## Changelog

* 2023-02-27: Created

## Context

As new releases of CometBFT Core come out, tendermint-rs should have a
viable way to support the versions of the protocol in use by the Cosmos
community. Hermes presents a particular challenge of needing to support
multiple different versions of the protocol within a single software agent
that performs IBC relaying across chains that are not necessarily using the
same CometBFT version.

Previously, it was feasible for tendermint-rs to support a single version of
the CometBFT Core protocol and expect all chain and relayer operators to
upgrade in sync. But now there is a number of established chains using
CometBFT 0.34 in production, and new chains using future versions of the
protocol are expected to be deployed and be interoperable with these existing
chains.

The approach to protocol evolution and versioning in CometBFT has not been
very rigorous to date. As a result, the 0.37 release has a number of breaking
changes in the RPC protocol format. Though the changes seem beneficial
in the long term, there is only an ad-hoc way to discover which version is used
by the node and select the appropriate encoding. It appears to be too
late in the development process to introduce significant changes to mitigate
this in the 0.37 release timeframe. A requirement to specify the
protocol version in chain configuration would complicate the deployment
experience for chain and relayer operators and should be avoided.

Another motivation for multi-version support is versioning of tendermint-rs
itself: previously, the semver compatibility of tendermint-rs releases was
tied with corresponding CometBFT Core version sequences in a none too obvious
way. It's desirable to decouple tendermint-rs versioning from that of CometBFT
Core, making it solely a matter of Rust crate API evolution. In the long term,
a single version of the tendermint-rs libraries should be able to support all
versions of CometBFT Core protocols that are relevant to the community.
The developers of CometBFT protocols, on there

## Decision

The tendermint-rs library API will be modified to provide support for
multiple versions of CometBFT Core protocol, at this moment being 0.34 and
0.37.

### tendermint-proto

The generated Rust files providing bindings for Tendermint protobuf defitions
will be emitted in two side-by-side modules, `tendermint::v0_34` and
`tendermint::v0_37`. All names under the latter module are also reexported under
the `tendermint` module, providing a low-change migration path for code bases
that used previous versions of the crate and the "default" import names for
those applications that only need to target the latest version.

### tendermint

The domain types are largely unchanged

### tendermint-rpc

A compatibility mode parameter is introduced for HTTP and WebSocket clients,
with the following type:

```rust
pub enum CompatMode {
    /// Use the latest version of the protocol (v0.37 in this release).
    Latest,
    /// Use v0.34 version of the protocol.
    V0_34,
}
```

The mode parameter is used to dynamically select the encoding used by a client.
By default, the clients use the latest protocol, but it's possible to specify
the compatibility mode at construction using the newly introduced configuration
API. In the HTTP client, it's also possible to switch the mode for an client
that has been already connected; this is useful for dynamic version discovery
using the `status` endpoint.

The `Client` trait is extended to support methods new to CometBFT 0.37,
which are emulated using older protocol endpoints when `V0_34` compatibility
mode is selected. There is no need to deprecate any methods. All data types
in the public client API are the domain types of `tendermint`, so the
difference in encoding is confined to the crate internals.

### tendermint-abci

This crate is not actively supported and we should only make minimal effort
to make it work with the proposed changes in a single chosen version of the
protocol.

## Open questions

* Should we migrate tendermint-abci to 0.37, freeze it at 0.34 and declare
  end of life, or commit to the same level of multi-version support as in
  other crates?

## Status

Proposed

## Consequences

### Positive

Interoperability with both 0.34 and 0.37 nodes is possible in a single built
executable and a single runtime process without much extra coding effort.

### Negative

The users of `tendermint-rpc` wishing to remain interoperable with 0.34 nodes
will have to configure the compatibility mode or add version discovery when
migrating to the release of the crate that introduces these changes.

Conversions from and to Protobuf message types in `tendermint-proto`
and the domain types in `tendermint` are no longer unambiguously resolved
by type inference and the types sometimes have to be explicitly annotated.

## References

* Implementation: [#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)