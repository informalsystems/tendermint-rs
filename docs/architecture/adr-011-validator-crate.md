# ADR 011: Validator Crate

## Changelog

* 2022-06-10: Created the ADR.

## Context

Tendermint provides an interface ("privval") for external processes
that validators deploy with different signing backends (e.g. YubiHSM).
In Tendermint 0.35 and beyond, the signer process is expected to be a gRPC server/service
that Tendermint connects to. Using `tonic-build`, the gRPC server interface
code can be generated as an extension to the existing `tendermint-proto` crate definitions.
The signing backend implementor could then possibly implement this interface 
from `tendermint-proto` as follows:

```rust
#[tonic::async_trait]
impl PrivValidatorApi for ValidatorSigningBackend
{
    async fn get_pub_key(
        &self,
        request: Request<RawPubKeyRequest>,
    ) -> Result<Response<RawPubKeyResponse>, Status> {
        /// ...
    }
    async fn sign_vote(
        &self,
        request: Request<RawSignVoteRequest>,
    ) -> Result<Response<RawSignedVoteResponse>, Status> {
        // ...
    }
    async fn sign_proposal(
        &self,
        request: Request<RawSignProposalRequest>,
    ) -> Result<Response<RawSignedProposalResponse>, Status> {
        // ...
    }
}
```

This implementation, however, needs to use the raw protobuf types,
so each implementor would need to duplicate the same common validation code.

## Decision

The `tendermint-validator` crate can be added to the tendermint-rs repository
that will help the developers with implementing Tendermint validator signing backends.
It will provide basic [tonic's Server](https://docs.rs/tonic/0.7.2/tonic/transport/struct.Server.html)
options (e.g. TLS) and the implementation of `PrivValidatorApi`. This implementation
can accept different signing backends that are implemented via two traits:

1. the signing one (that e.g. communicates with HSM):
```rust
#[tonic::async_trait]
pub trait SignerProvider {
    type E: std::error::Error;
    async fn sign(&self, signable_bytes: &[u8]) -> Result<Signature, Self::E>;
    async fn load_pubkey(&self) -> Result<PublicKey, Self::E>;
}
```

2. the state storage one that persists the validator state (in a file or whatever
makes sense in the signing context, e.g. write to CPU monotonic counters):

```rust
#[tonic::async_trait]
pub trait ValidatorStateProvider {
    type E: std::error::Error;
    async fn load_state(&self) -> Result<consensus::State, Self::E>;
    async fn persist_state(&mut self, new_state: &consensus::State) -> Result<(), Self::E>;
}
```

Besides checking the chain ids from requests and calling corresponding providers,
the implementation can also do the following common validation steps:

- use the existing domain types from the `tendermint` crate to validate the incoming requests;
- check the signing requests' heights against the optional maximum;
- check the signing requests against the last validator state and persist the new states after successful signing operations.

For the validator state checking, the existing consensus `State` can be extended to have
conversions from the signing requests as well as 
[`PartialOrd` implementation](https://github.com/iqlusioninc/tmkms/blob/main/src/chain/state.rs#L70).

The crate can also contain an example "software signer" backend implementation of the provider traits
that stores the signing key in-memory and uses text files for the validator state persistence.

## Status

Accepted

## Consequences

### Positive

* Reduced code duplication
* Easier implementation of new signing backends

### Negative

* The need to use async functions due to `tonic`'s gRPC implementation

### Neutral

* One more crate

## References

* https://github.com/informalsystems/tendermint-rs/issues/1134
* https://github.com/informalsystems/tendermint-rs/pull/1137
