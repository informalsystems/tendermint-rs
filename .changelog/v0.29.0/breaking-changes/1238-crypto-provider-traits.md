- `[tendermint]` Make implementations of cryptographic primitives replaceable
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * Provide a `Sha256` trait in module `crypto` and make digest hashing
    implementations available through it.
  * Provide a `Verifier` trait in module `crypto::signature` to enable
    alternative implementations of signature verification available through it.
    An `Error` enum is defined in the same module, representing the error cases
    that can arise in the implementation in a deliberately opaque way.
  * The module `crypto::default` provides pure Rust implementations of the
    cryptographic traits. The module is made available by a
    new `rust-crypto` feature, enabled by default.
  * `merkle::simple_hash_from_byte_vectors` is made generic over an
    implementation of the new `MerkleHash` trait. Implementations for
    Rust-Crypto conformant digest objects and the non-incremental
    `crypto::Sha256` API are provided in the crate.
  * The `Header::hash` and `ValidatorSet::hash` methods are gated by the
    `rust-crypto` feature. Generic hashing methods not dependent on
    the default crypto implementations are added for both types,
    named `hash_with`.
  * Conversions to `account::Id` and `node::Id` from `PublicKey` and
    curve-specific key types are gated by the `rust-crypto` feature.
  * The `validator::Info::new` method is gated by the `rust-crypto` feature.
  * Remove a deprecated constant `signature::ED25519_SIGNATURE_SIZE`.

- `[tendermint-light-client-verifier]` Changes for the new Tendermint crypto API
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * The `rust-crypto` feature, enabled by default, guards the
    batteries-included implementation types: `ProdVerifier`, `ProdPredicates`,
    `ProdVotingPowerCalculator`.
  * Remove the `operations::hasher` API (`Hasher` and `ProdHasher`),
    made unnecessary by the new crypto abstractions in the `tendermint` crate.
  * The `VerificationPredicates` trait features a `Sha256` associated type
    to represent the hasher implementation, replacing the `&dyn Hasher`
    parameter passed to methods.
  * Change the type of the `VerificationErrorDetail::FaultySigner` field
    `validator_set` to `ValidatorSet`. This removes a hasher dependency from
    `CommitValidator`, and `ProdCommitValidator` is now an empty dummy type.

- `[tendermint-light-client]` Changes for the new Tendermint crypto API
  ([#1238](https://github.com/informalsystems/tendermint-rs/pull/1238)).
  * The `rust-crypto` feature enables the default crypto implementations,
    and is required by the `rpc-client` and `unstable` features.
    `ProdForkDetector` is guarded by this feature, and is made a specific
    type alias to the hasher-generic `ProvidedForkDetector` type.
  * `LightClientBuilder` gets another type parameter for the Merkle hasher.
    Its generic constructors lose the `Hasher` parameter.
