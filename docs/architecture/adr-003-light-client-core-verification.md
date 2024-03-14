# ADR 003: Light Client Core Verification

## Changelog

This used to be part of ADR-002 but was factored out.

- 2020-02-09: Realignment with latest code and finalize.
- 2020-01-22: Factor this new ADR out of ADR-002 and reduce to just the core
verification component.

## Status

Approved. Implemented.

## Context

The high level context for the light client is described in
[ADR-002](adr-002-light-client-adr-index.md).

For reference, a schematic of the light node is below:

![Light Node Diagram](assets/light-node.png).

Here we focus on the core verification library, which is reflected in the
diagram as the "Light Client Verifier" and "Bisector".

The light node is subjectively initialized, and then attempts to sync to a most
recent height by skipping straight to it. The verifier is the core logic
to check if a header can be trusted and if we can skip to it.
If the validator set has changed too much, the header can't be trusted,
and we'll have to request intermediate headers as necessary to sync through the validator set changes.

Note the verifier should also work for IBC, though in IBC bisection can't be performed directly
since blockchains can't make HTTP requests. There, bisection is expected to be done by an external relayer
process that can make requests to full nodes and submit the results to an IBC enabled chain.

The library should lend itself smoothly to both use cases and should be difficult to use incorrectly.

The core verification operates primarily on data types that are already implemented
in tendermint-rs (headers, public keys, signatures, votes, etc.). The crux of it
is verifying validator sets by computing their merkle root, and verifying
commits by checking validator signatures. The particular data structures used by
Tendermint have considerably more features/functionality than needed for this,
hence the core verification library should abstract over it.

Here we describe the core verification library, including:

- traits
- public functions for IBC and light nodes
- implementations of traits for the existing Tendermint data-structures

## Decision

The implementation of the core verification involves two components:

- a new `light` crate, containing traits and functions defining the core verification
  logic
- a `light-impl` module within the `tendermint` crate, containing implementations of
  the traits for the tendermint specific data structures

The `light` crate should have minimal dependencies and not depend on
`tendermint`. This way it will be kept clean, easy to test, and easily used for
variations on tendermint with different header and vote data types.

The light crate exposes traits for the block header, validator set, and commit
with the minimum information necessary to support general light client logic.

According to the specification, the key functionality the light client
verification requires is to determine the set of signers in a commit, and
to determine the voting power of those signers in another validator set.
Hence we can abstract over the lower-level detail of how signatures are
validated.

### Header

A Header must contain a height, time, and the hashes of the current and next validator sets.
It can be uniquely identified by its hash:

```rust
pub trait Header {
    fn height(&self) -> Height;
    fn bft_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    fn hash(&self) -> Hash;
}
```

### Commit

A commit in the blockchain contains the underlying signatures from validators, typically in
the form of votes. From the perspective of the light client,
we don't care about the signatures themselves.
We're only interested in knowing the total voting power from a given validator
set that signed for the given block. Hence we can abstract over underlying votes and
signatures and just expose a `voting_power_in`, as per the spec:

```rust
pub trait Commit {
    type ValidatorSet: ValidatorSet;

    fn header_hash(&self) -> Hash;
    fn validate(&self, &Self::ValidatorSet) -> Result<(), Error>;
    fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error>;
}
```
The `header_hash` is the header being committed to,
and `validate` performs Commit-structure-specific validations,
for instance checking the number of votes is correct, or that they are
all for the right block ID.

Computing `voting_power_in` require access to a validator set
to get the public keys to verify signatures. But this specific relationship
between validators and commit structure isn't the business of the `light` crate;
it's rather how a kind of Tendermint Commit structure implements the `light` traits.
By making ValidatorSet an associated type of Commit, the light crate doesn't
have to know much about validators or how they relate to commits, so the ValidatorSet
trait can be much smaller, and we never even define the concept of an individual
validator.

Note this means `Commit` is expected to have access to the ids of the validators that
signed, which can then be used to look them up in their ValidatorSet implementation.
This is presently true, since Commit's contain the validator addresses
along with the signatures. But if the addresses were removed, for instance to
save space, the Commit trait would need access to the relevant validator set to
get the Ids, and this validator set may be different than the one being passed
in `voting_power_in`.

By abstracting over the underlying vote type, this trait can support
optimizations like batch verification of signatures, and the use of
aggregate signatures instead of individual votes.
So long as it can be determined what voting power of a given validator set
signed correctly for the commit.

The method `voting_power_in` performs the underlying signature verifications.
It should return an error if any of them fail or are for the wrong header hash.

Note the specification introduces a `signers(commit) -> validators` method that
returns the list of validators that signed a commit. However, such a method
would require access to the underlying validator set in order to verify the
commits, and it is only ever used in computing `voting_power_in`. Hence, we
dispense with it here in favour of a `voting_power_in` that operates on a
`Commit` and `ValidatorSet`. However, this also means that ValidatorSet will
need to expose facilities for determining whether a validator signed correctly in
order for implementations to make use of it to compute `voting_power_in`.

Note also that in Tendermint, commits are for a particular block ID, which
includes both a header hash and a "parts set hash". The latter is completely
irrelevant to the light client, and can only be verified by downloading the full
block. Hence it is effectively ignored here. It would be great if Tendermint
could disentangle commits to the proposal block parts for gossip (ie. the parts
set hash) from commits to the header itself (ie. the header hash), but that's
left for the future.

For more background on implementation of Tendermint commits and votes, see:
- [ADR-025](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-025-commit.md)
- [Validator Signing Spec](https://github.com/tendermint/tendermint/blob/main/docs/spec/consensus/signing.md)
- [Tendermint consensus specification](https://arxiv.org/abs/1807.04938)

### Validator Set

A validator set has a unique hash which must match what's in the header.
It also has a total power used for determining if the result of `voting_power_in` is greater
than a fraction of the total power. 

ValidatorSet is implemented as an associated type of Commit, where it's
necessary to compute `voting_power_in`, so the underlying implementation must
have some way to determine the voting power of the validators that signed,
since voting power is not found in the commit itself.

Note we don't need to define individual validators since all the details of how validators relates to commits
is encapsulated in the Commit.

```rust
pub trait ValidatorSet {
    fn hash(&self) -> Hash;
    fn total_power(&self) -> u64;
}
```

### State

According to the spec, the light client is expected to have a store that it can
persist trusted headers and validators to. This is necessary to fetch the last trusted
validators to be used in verifying a new header, but it's also needed in case
any conflicting commits are discovered and they need to be published to the
blockchain. That said, it's not needed for the core verification, so we don't
include it here. Users of the `light` crate like a light node decide how to
manage the state. We do include some convenience structs:

```rust
pub struct SignedHeader<C, H> 
where 
    C: Commit, 
    H: Header,
{
    commit: C,
    header: H,
}

pub struct TrustedState<C, H>{
where 
    C: Commit, 
    H: Header,
{
    last_header: SignedHeader<C, H>,
    validators: C::ValidatorSet,
}
```

Here the trusted state combines both the signed header and the validator set,
ready to be persisted to some external store.

### TrustThreshold

The amount of validator set change that occur when skipping to a higher height depends on the
trust threshold, as per the spec. Here we define it as a trait that encapsulated the math of what percent
of validators need to sign:

```rust
pub trait TrustThreshold: Copy + Clone {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool;
}
```

We provide a conenvient implementation that takes a numerator and a denominator. The default is of course 1/3.

### Requester

The light node needs to make requests to full nodes during bisection for intermediate signed headers and validator sets:

```rust
pub trait Requester<C, H>
where
    C: Commit,
    H: Header,
{
    fn signed_header(&self, h: Height) -> Result<SignedHeader<C, H>, Error>;
    fn validator_set(&self, h: Height) -> Result<C::ValidatorSet, Error>;
}
```

In practice, this can be implemented as a Tendermint RPC client making requests
to the `/commit` and `/validators` endpoints of full nodes.
For testing, the Requester can be implemented by JSON files.

### Verification

Both IBC and full node syncing have to perform a common set of checks:

- validate the hashes
- if the header is sequential, validate the next validator set
- if the header is not sequential, check if the trust threshold is reached
	- this uses `voting_power_in` with a validator set that may be different from
the one that actually created the commit.
- check that +2/3 of the validators signed
	- this uses `voting_power_in` with the actual validator set

These are implemented in a common function, `verify_single_inner`:

```rust
fn verify_single_inner<H, C, L>(
    trusted_state: &TrustedState<C, H>,
    untrusted_sh: &SignedHeader<C, H>,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: L,
) -> Result<(), Error>
```

Note however that light client security model is highly sensitive to time, so the public functions
exposed for IBC and bisection, which will call `verify_single_inner`, must take a current time
and check we haven't expired.

For IBC, since it can't make its own requests, the public function just takes the untrusted state
in full, and return it as a TrustedState if it verifies:

```rust
pub fn verify_single<H, C, T>(
    trusted_state: TrustedState<C, H>,
    untrusted_sh: &SignedHeader<C, H>,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: T,
    trusting_period: &Duration,
    now: &SystemTime,
) -> Result<TrustedState<C, H>, Error>
```

For the light node, we pass in a Requester, and specify a height we want to sync to.
It will fetch that header and try to verify it using the skipping method,
and will run a bisection algorithm to recursively request headers of lower height
as needed. It returns a list of headers it verified along the way:

```rust
pub fn verify_bisection<C, H, L, R>(
    trusted_state: TrustedState<C, H>,
    untrusted_height: Height,
    trust_threshold: L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
) -> Result<Vec<TrustedState<C, H>>, Error>
```

### Implementing Traits

The core `light` traits can be implemented by the Tendermint data structures and
their variations. For instance, v0.33 of Tendermint Core introduced a breaking
change to the Commit structure to make it much smaller. We can implement the
`light` traits for both versions of the Commit structure.

The `light` abstractions also facilitate testing, as complete Tendermint data structures
are not required to test the light client logic, only
the elements it cares about. This means we can implement mock commits and validators,
where validators are just numbered 0,1,2... and both commits and validators are
simply lists of integers representing the validators that signed or are in the
validator set. This aligns closely with how these structures are represented in
the [TLA+
spec](https://github.com/interchainio/verification/blob/develop/spec/light-client/Blockchain.tla).

While this provides a lot of flexibility in mocking out
the types, we must be careful to ensure they match the semantics of the actual
Tendermint types, and that we still test the verification logic sufficiently for
the actual types.

### Other Validation

Some fields in the header are left explicitly unvalidated as they have minimal bearing on the correctness of the light client.
These include:

- LastCommitHash
	- In the skipping case, it's not possible to verify the header refers to the correct previous block without reverting to the sequential case. So in the sequential case, we don't validate this either. If it's incorrect, in indicates the validators are misbehaving, though we can only detect it as a light client if there's a fork.
- BlockID
	- As mentioned, this includes a merkle root of the entire block and is not verifiable without downloading the whole block, which would defeat the purpose of the light client!
- Time
	- Verifying the time would require us to download the commit for the previous block, and to take the median of the timestamps from that commit. This would add significant overhead to the light client (an extra commit to validate for every block!). If the time is incorrect, in indicates that the validators are explicitly violating the protocol rules in a detectable way which full nodes should detect in the first place and shouldn't forward to light clients, so there would probably be bigger issues at foot.

There are likely a few other instances of things the light client is not validating that it in theory could but likely indicate some larger problem afoot that the client can't do anything about anyways. Hence we really only focus on the correctness of commits and validator sets and detecting forks!


## Consequences

### Positive

- Clear separation between verification logic and the actual data types
- Traits can be easily mocked without real keys/signatures
- Public interface is hard to misuse.

### Negative

- Abstract traits requires more coding, more opportunity for bugs in trait implementation

### Neutral

- Certain validity checks are omitted since they have little bearing
