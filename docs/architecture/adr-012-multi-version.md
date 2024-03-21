# ADR 012: Support for multiple versions of CometBFT Core

## Changelog

* 2023-03-02: First draft

## Context

As new releases of CometBFT come out, tendermint-rs should have a
viable way to support the versions of the protocol in use by the Cosmos
community. Hermes presents a particular challenge of needing to support
multiple different versions of the protocol within a single software agent
that performs IBC relaying across chains that are not necessarily using the
same CometBFT version.

Previously, it was feasible for tendermint-rs to support a single version of
the CometBFT protocols and expect all chain and relayer operators to
upgrade their software in sync. But now there is a number of established chains
using CometBFT 0.34 in production, and new chains using future versions of the
protocol are expected to be deployed into the shared Cosmos ecosystem
so that relaying is possible between the old and the new chains.

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
tied with corresponding CometBFT version sequences in a none too obvious way.
It's desirable to decouple tendermint-rs versioning from that of CometBFT,
making it solely a matter of Rust crate API evolution. In the long term,
a single version of the tendermint-rs libraries should be able to support all
versions of CometBFT protocols that are relevant to the community.
The developers of CometBFT protocols should take versioning practices into use
for future revisions that support backward compatibility on the source code level.

## Decision

The tendermint-rs library API will be modified to provide support for
multiple versions of CometBFT APIs, at this moment being 0.34 and 0.37.

### tendermint-proto

The generated Rust files providing bindings for Tendermint protobuf definitions
will be emitted in two side-by-side modules, `tendermint::v0_34` and
`tendermint::v0_37`. All names under the latter module are also reexported under
the `tendermint` module, providing a low-change migration path for code bases
that used previous versions of the crate, and the "default" import paths for
those applications that only need to target the latest version.

### tendermint

The domain types are largely unchanged, except where additional fields are
needed to support version 0.37. Some new types are added as required by the
new version. Conversions to and from Protobuf types defined in
`tendermint-proto` are provided for both `tendermint::v0_34` and
`tendermint::v0_37` generated types, except where the types are new to 0.37.

### tendermint-rpc

A compatibility mode parameter is introduced for HTTP and WebSocket clients,
with the following type:

```rust
pub enum CompatMode {
    /// Use the v0.37 version of the protocol.
    V0_37,
    /// Use v0.34 version of the protocol.
    V0_34,
}
```

The mode parameter is used to dynamically select the encoding used by a client.
By default, the clients use the latest protocol (designated by the associated
const function `CompatMode::latest()`), but it's possible to specify
the compatibility mode at construction using the newly introduced configuration
API. In the HTTP client, it's also possible to switch the mode for a client
that has been already connected; this is useful for dynamic version discovery
using the `status` endpoint.

The `Client` trait is extended to support methods new to CometBFT 0.37,
which are emulated using older protocol endpoints when `V0_34` compatibility
mode is selected. There is no need to deprecate any methods. All data types
in the public client API are the domain types of `tendermint`, so the
difference in encoding is confined to the crate internals.

Protocol version discovery is not implemented in the library in a way that
would be invisible to the API user. To discover the Tendermint version, a client
needs to make a `status` request and process the version data from the response.
The format of the response is not divergent between the supported protocol
versions, which is why this should succeed regardless of the compatibility mode
initially selected. As the RPC client API represents individual endpoint requests,
it would be wrong to have the implementation perform a hidden RPC roundtrip to
discover the version when needed, and have the client exhibit interior
mutability that is otherwise not needed. In the future, a more formalized way
to discover the protocol version in use should be provided, so this ad-hoc
approach is expected to be eventually deprecated.

#### Examples

Connecting to the WebSocket endpoint with a specified compatibility mode:

```rust
use tendermint_rpc::client::{CompatMode, WebSocketClient};

// ...

let client = WebSocketClient::builder(rpc_url)
    .compat_mode(CompatMode::V0_34)
    .build()
    .await?;
```

Discovery of the RPC compatibility mode while preserving the HTTP connection:

```rust
use tendermint_rpc::client::{Client, CompatMode, HttpClient, HttpClientUrl};
use tendermint_rpc::error::Error;

async fn rpc_client_with_version_discovery<U>(url: U) -> Result<HttpClient, Error>
where
    U: TryInto<HttpClientUrl, Error = Error>,
{
    let mut rpc_client = HttpClient::new(url)?;
    let status = rpc_client.status().await?;
    let compat_mode = CompatMode::from_version(status.node_info.version)?;
    rpc_client.set_compat_mode(compat_mode);
    Ok(rpc_client)
}
```

### tendermint-abci

This crate is not actively supported and we should only make the minimal effort
to update it to the 0.37 version of ABCI. No backward compatibility with 0.34
or multi-version support is to be implemented; the consumers should be steered
towards [tower-abci](https://github.com/penumbra-zone/tower-abci).

## Status

Proposed

## Consequences

### Positive

RPC interoperability with both 0.34 and 0.37 nodes is possible in a single built
executable and a single runtime process without much extra coding effort.

Consumers of other tendermint-rs crates will get support for both 0.34 and 0.37
versions of the protobuf messages and be able to use either or both.

### Negative

The consumers of `tendermint-rpc` wishing to remain interoperable with 0.34 nodes
will have to configure the compatibility mode or add version discovery when
migrating to the release of the crate that introduces these changes.

Conversions from and to Protobuf message types in `tendermint-proto`
and the domain types in `tendermint` are no longer unambiguously resolved
by type inference and the types sometimes have to be explicitly annotated.

The existing consumers of `tendermint-abci` who wish to remain on 0.34
will be effectively stranded on the previous semver break release of the crate
(0.29.x as of this writing).

## References

* Implementation: [#1193](https://github.com/informalsystems/tendermint-rs/pull/1193)
