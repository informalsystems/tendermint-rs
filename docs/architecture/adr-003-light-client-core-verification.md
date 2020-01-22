# ADR 003: Light Client Core Verification

## Changelog

This used to be part of ADR-002 but was factored out.

2020-01-22: Factor this new ADR out of ADR-002 and reduce to just the core
verification component.

## Status

Approved. Implemented.

## Context

The high level context for the light client is described in
[ADR-002](adr-002-light-client-adr-index.md).

For reference, a schematic of the light node is below:

![Light Node Diagram](assets/light-node.png).

Here we focus on the core verification library, which is reflected in the
diagram as the "Light Client Verifier". 

The core verification operates primarily on data types that are already implemented
in tendermint-rs (headers, public keys, signatures, votes, etc.). The crux of the 
library is verifying validator sets by computing their merkle root, and verifying
commits by checking validator signatures. The particular data structures used by
Tendermint have considerably more features/functionality than needed for this, 
- hence the core verification library should abstract over it.

Here we describe the core verification library, including:

- traits and methods
- implementations of traits for the existing Tendermint data-structures

## Decision

### Verifier

Most of the light client logic resides in the verifier, where commits for a
header are actually verified. In order to abstract over all the data in the
Tendermint data structures, we use a set of traits that provide only the
information absolutely necessary for the light client. 

According to the specification, the key functionality the light client
verification requires is to determine the set of signers in a commit, and
to determine the voting power of those signers in another validator set.
Hence we can abstract over the lower-level detail of how signatures are
validated.

#### Header

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

#### Commit

A commit contains the underlying signatures from validators, typically in 
the form of votes, but from the perspective of the light client, 
we don't care about the signatures themselves.
We're only interested in knowing the total voting power from a given validator
set that signed for the given block. Hence we can abstract over underlying votes and
signatures and just expose a `voting_power_in`, as per the spec.

The `header_hash` indicates the header being committed to, 
and `vote_len` methods are for simple validity checks.

```rust
pub trait Commit {
    fn header_hash(&self) -> Hash;
    fn votes_len(&self) -> usize;

    fn voting_power_in<VS>(&self, vals: &VS) -> Result<u64, Error> 
        where VS: ValidatorSet;
}
```

By abstracting over the underlying vote type, this trait can support
optimizations like batch verification, and changes to the
underlying commit structure, like using agreggate signatures instead of
individual votes. So long as it can be determined what voting power of a given validator set
signed correctly for the commit.

The method `voting_power_in` performs the underlying signature verifications.
It should return an error if any of them fail or are for the wrong header hash.

Note the specification introduces a `signers(commit) -> validators` method that
returns the list of validators that signed a commit. However, such a method
would require access to the underlying validator set in order to verify the
commits, and it is only ever used in computing `voting_power_in`. Hence, we
dispence with it here in favour of a `voting_power_in` that operates on a
`Commit` and `ValidatorSet`. However, this also means that ValidatorSet will
need to expose facilities for determining wheter a validator signed correctly in
order for implementations to make use of it to compute `voting_power_in`.

Note also that in Tendermint, commits are for a particular block ID, which
includes both a header hash and a "parts set hash". The latter is completely
irelevant to the light client, and can only be verified by downloading the full
block. Hence it is effectively ignored here. It would be great if Tendermint
could disentangle commits to the proposal block parts for gossip (ie. the parts
set hash) from commits to the header itself (ie. the header hash), but that's
left for the future.


For more background on implementation of Tendermint commits and votes, see:
- [ADR-025](https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-025-commit.md)
- [Validator Signing Spec](https://github.com/tendermint/tendermint/blob/master/docs/spec/consensus/signing.md)
- [Tendermint consensus specification](https://arxiv.org/abs/1807.04938)

#### Validator Set

A validator set has a unique hash which must match what's in the header.
It exposes a length which should match the commit's `vote_len`. It also has a
total power used for determining if the result of `voting_power_in` is greater
than a fraction of the total power. 

Most importantly, a validator set trait must facilitate computing `voting_power_in` 
for a Commit, which means it needs to expose some way to determine the voting power 
of the validators that signed. Note, however, that this functionality is not used 
by the light client logic itself - it's only used by the implementation of
Commit within the `voting_power_in` method. Hence, further abstraction may be
warranted to eliminate the need for such a method. For instance, perhaps
ValidatorSet could be an associated type of Commit, and then implementations of
Commit would specify their ValidatorSet and how they verify signatures.

In the meantime, we use a ValidatorSet trait that exposes a lookup method 
for fetching validators by their Id. An associated validator type (below) can then expose 
it's voting power and a method for verifying signatures. Implementations of Commit can use this to
determine the voting power of the validators that signed. 

```rust
pub trait ValidatorSet {
    type Validator: Validator;

    fn hash(&self) -> Hash;
    fn total_power(&self) -> u64;
    fn len(&self) -> usize;

    fn validator(&self, val_id: Id) -> Option<Self::Validator>;
}
```

Note that this also assumes `Commit` has access to the Id of the validators that
signed. This is presently true, since Commit's contain the validator addresses
along with the signatures. But if the addresses were removed, for instance to
save space, the Commit trait would need access to the relevant validator set to 
get the Ids, and this validator set may be different than the one being passed
in `voting_power_in`! This again suggests the need for a better abstraction, and
more closely associating the validator lookup and signature verification
functionality with the Commit trait.

#### Validator

A validator contributes a positive voting power if a message was correctly
signed by it, otherwise it contributes 0. We could represent this with a single
method that returns either 0 or the voting power, but it's probably clearer with
two methods:

```rust
pub trait Validator {
    fn power(&self) -> u64;
    fn verify_signature(&self, sign_bytes: &[u8], signature: &[u8]) -> bool;
}
```

This trait is needed for the validator lookup method in the ValidatorSet, but as
per the note above, if that method can be eliminated, so can this trait.

### State

According the spec, the light client is expected to have a store that it can
persist trusted headers and validators to. This is necessary to fetch the last trusted
validators to be used in verifying a new header, but it's also needed in case
any conflicting commits are discovered and they need to be published to the
blockchain. While it's not needed for the core verification, which can be
assumed pure, it is needed for a fully working client. Hence we introduce 
additional traits:


```rust
pub trait SignedHeader {
    type Header: Header;
    type Commit: Commit;
    
    fn header(&self) -> &Self::Header;
    fn commit(&self) -> &Self::Commit;
}

pub trait TrustedState {
    type LastHeader: SignedHeader; 
    type ValidatorSet: ValidatorSet;

    fn new(last_header: Self::LastHeader, vals: Self::ValidatorSet) -> Self;
    
    fn last_header(&self) -> &Self::LastHeader; // height H-1
    fn validators(&self) -> &Self::ValidatorSet; // height H
}

pub trait Store {
    type State: TrustedState;

    fn add(&mut self, state: &Self::State) -> Result<(), Error>;
    fn get(&mut self, h: Height) -> Result<&Self::State, Error>;
}
```

Here the trusted state combines both the signed header and the validator set,
and the store persists it all together under the relevant height.


### Implementing Traits

The tendermint-rs library includes Header, Vote, Validator, ValidatorSet, and
Commit data types. However, rather than use these types directly, the light 
client library is written more abstractly to use traits that contain only the
necessary information and functionality from these more concrete types. While this may turn out to
be an unecessarily eager abstraction (as we do not forsee alternative
implementations of these traits in the short term), it does provide a very clear
depiction of what is required for light client verification, and surfaces certain
design issues in the underlying Tendermint blockchain (eg. the `BlockID` issue
referenced above).

This abstraction may also facilitate testing, as we will not need to
generate complete Tendermint data structures to test the light client logic, only
the elements it cares about. While this provides a lot of flexibility in mocking out
the types, we must be careful to ensure they match the semantics of the actual
Tendermint types, and that we still test the verification logic sufficiently for
the actual types.

### Verification

Verification comes in two forms: full verification and "trusting" verification.
The former checks whether the commit was correctly signed by its validator set.
The latter checks whether +1/3 of a trusted validator set from the past signed a
future commit.

#### Full Verification

Since we know the validators for a commit,
we can check that the number of validators matches the number of votes in the commit.
In this case, `voting_power_in` uses a validator set that is the same as the one
that created the commit.

So we can have a function like:

```rust
fn verify_commit_full<V, C>(vals: V, commit: C) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
{
```

#### "Trusting" Verification

To do skipping verification (ie. the "trusting method"), 
we have to check if +1/3 of validators at some past height signed the commit, 
before we can check if +2/3 of the validators for the current height signed.
However, as per the spec, we also add a `trust_level` to the +1/3, to modulate 
how willing we are to skip, potentially requiring even more than +1/3.

So we can have a function like:

```rust
fn verify_commit_trusting<V, C, L>(vals: V, commit: C, trust_level: L) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
    L: TrustThreshold,
{
```

In this case, `voting_power_in` uses a validator set that is distinct from the
one that actually created the commit. Specifically, it uses the last trusted
validator set, and attempts to see if we can trust the new validator set.

If this function passes, then we can trust the new validator set, and we can 
call `verify_commit_full` with the new (and now correct) validator set to determine 
if they signed correctly.

Note we introduce the `TrustThreshold` trait, which exposes a single method
`is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64)` with
a default implementation that returns `signed_voting_power * 3 >
total_voting_power` (ie. +1/3).

#### Validation

Most of the above is about checking the signatures and voting power in commits, 
but we also need to perform other validation checks, 
like that the validator set hashes match what are in the header, and the light
client's trusted state actually hasn't expired. Pure functions for all of these
checks should be provided.

Some things are left explicitly unvalidated as they have minimal bearing on the correctness of the light client.
These include:

- LastCommitHash
	- In the skipping case, it's not possible to verify the header refers to the correct previous block without reverting to the sequential case. So in the sequential case, we don't validate this either. If it's incorrect, in indicates the validators are misbehaving, though we can only detect it as a light client if there's a fork.
- BlockID
	- As mentioned, this includes a merkle root of the entire block and is not verifiable without downloading the whole block, which would defeat the purpose of the light client!
- Time
	- Verifying the time would require us to download the commit for the previous block, and to take the median of the timestamps from that commit. This would add significant overhead to the light client (an extra commit to validate for every block!). If the time is incorrect, in indicates that the validators are explicitly violating the protocol rules in a detectable way which full nodes should detect in the first place and shouldn't forward to light clients, so there would probably be bigger issues at foot.

There are likely a few other instances of things the light client is not validating that it in theory could but likely indicate some larger problem afoot that the client can't do anything about anyways. Hence we really only focus on the correctness of commits and validator sets and detecting forks!

### Lite Node Sync

Finally, to put it all together, we define a `verify_header` function which
attempts to sync to some given height:

```rust
pub fn verify_header<TS, SH, VS, L, S, R>(
    height: Height, trust_threshold: &L, trusting_period: &Duration,
    now: &SystemTime, req: &R, store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = VS>,
    SH: SignedHeader,
    VS: ValidatorSet,
    L: TrustThreshold,
    S: Store<State = TS>,
    R: Requester<SignedHeader = SH, ValidatorSet = VS>,
{
```

This function gets the latest state from the store, fetches the header for the
given height from a peer, and attempts to verify the header using the skipping
method, and running a bisection algorithm to recursively request headers of
lower height as needed. Every time it verifies a header, it updates the store.

As new headers are added to the store, we need to alert the detection module, so
it can start searching for conflicts. The coupling between the verification and
detection modules should be minimized. For now, we may assume the detection
module continuously polls the store for new headers, but in the future we may
use more explicit communication eg. via a channel.

Note this function is specific to the light node (rather than IBC) as it
implements the bisection algorithm directly and assumes the client has the
ability to make requests (where in the IBC case, we do not).

## Status

Proposed

## Consequences

### Positive

- Implements the light node!
- Simple peering strategy
- Clear separation between verification logic and the actual data types

### Negative

- Abstract traits requires more coding, more opportunity for bugs in trait implementation

### Neutral

- Certain validity checks are ommitted since they have little bearing
