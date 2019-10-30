# ADR 002: Lite Client

## Context

Here we describe the architecture for a Tendermint lite node in Rust,
including a core lite client library. While the lite client library provides the
essential verification logic according to the [english](TODO) and [TLA+](TODO)
specs, the lite node uses the lite client library to sync its local state to the
latest state of the blockchain using a list of full nodes. In addition to lite
nodes running on client machines making rpc queries to full nodes, the core lite
client library should also be usable by IBC handlers on blockchains receiving
data from relayers. While this document only covers the case of the lite node,
it's important to have clear separation of concerns so that the library can be
reused in the IBC context.

Most of the necessary data types for the lite client already exist (pubkey/signature, blockid,
vote), with support for serialization/deserialization. The crux of the lite
client is verifying validator sets by computing their merkle root, and verifying
commits by checking validator signatures. The particular data structures used by
Tendermint have considerably more features/functionality, much of which is not
needed for the lite client protocol - hence it can be abstracted away from
the core verification logic.

In addition to the core verification logic, the lite node needs a way to
receive data from full nodes, to detect conflicting information, and to report
on conflicts. While there are many ways for a full node to provide bad
information, what we're really looking for is misbehaviour by the validators,
which is reflected in conflicting commits (ie. commits for different blocks at
the same height). 

In what follows we outline the following components of the lite client software:

- core lite client verification library and traits
- how Tendermint data-structures implement the core lite client traits
- how a lite node requests information from full nodes and detects conflicts

Note that the architecture of IBC is out of scope, suffice it to say that the
core libraries (ie. the verification library and the implementation of core
traits by the Tendermint types) will be re-usable by IBC.

## Decision

Below is a schematic of the components of a lite node:

![Lite Client Diagram](assets/adr-002-image.png)

Essentially, the node is initialized with a trusted header for some height H-1
(call this header[H-1]), a validator set for height H (call this vals[H]),
and a list of peers. For the sake of simplicity, one of the peers is selected as the "primary", while the
rest are considered "backups". Most of the data is downloaded from the primary,
and double checked against the backups.

The state is considered "expired" if the difference between the current time and
the time from the trusted header is greater than a configurable "trusting
period". If at any point the state is expired, the node should log an error and
exit - it's needs to be manually reset.

We take up the components of the diagram in term.

### Manager

For lack of a better name. The Manager co-ordiantes the syncing and is the
highest level component. We consider two approaches to syncing the lite node: sequential and skipping.

#### Sequential Sync

Inital state: 
    - time T
    - height H 
    - header[H-1]
    - vals[H]

Here we describe the happy path:

1) Request header[H], commit[H], and vals[H+1] from the primary, and check that they are well formed and from the correct height
2) Pass header[H], commit[H], vals[H], and vals[H+1] to the verification library, which
  will:
    - check that vals[H] and vals[H+1] are correctly reflected in the header
    - check that the commit is for the header
    - check that +2/3 of the validators correctly signed the header hash
3) Request header[H] from each of the backups and check that they match header[H] received from the primary
4) Update the state with header[H] and vals[H+1], and increment H
5) return to (1)

If (1) or (2) fails, mark the primary as bad and select a new peer to be the
primary.

If (3) returns a conflicting header, verify the header by requesting the
corresponding commit and running the verification of (2). If the verification
passes, there is a fork, and evidence should be published so the validators get
slashed. We leave the mechanics of evidence to a future document. For now, the
lite client will just log and error and exit. If the verification fails, it
means the backup that provided the conflict is bad and should be removed.

#### Skipping Sync

Skipping sync is essentially the same as sequential, except for a few points:

- instead of verifying sequential headers, we attempt to "skip" ahead to the
  full node's most recent height
- skipping is only permitted if the validator set has not changed too much - ie.
  if +1/3 of the last trusted validator set has signed the commit for the height we're attempting to skip to
- if the validator set changes too much, we "bisect" the height space,
  attempting to skip to a lower height, recursively. 
- in the worst case, the bisection takes us to a sequential height

### Requester

The requester is simply a Tendermint RPC client. It makes requests to full
nodes. It uses the `/commit` and `/validators` endpoints to get signed headers
and validator sets for relevant heights. It may also use the `/status` endpoint
to get the latest height of the full node (for skipping verification).

### Detect

The detection module is really just about checking if any of the backup nodes
are reporting conflicting information. It requests headers from each backup node
and compares them with a verified header from the primary. If there is a
conflict, it attempts to verify the conflicting header via the verifier. If it
can be verified, it indicates an attack on the lite clients that should be
punishable. The relevant information (ie. the two conflicting commits) are
passed to the publisher.

### Publisher

For now, the publisher just logs an error, write the conflicting commits to a
file, and exits. We leave it to a future document to describe how this
information can actually be published to the blockchain so the validators can be
punished. Tendermint may need to expose a new RPC endpoint to facilitate this.

### Address Book

For now this is a simple list of HTTPS addresses corresponding to full nodes
that the node connects to. One is randomly selected to be the primary, while
others serve as backups. It's essential that the lite node connect to at least
one correct full node in order to detect conflicts in a timely fashion. We keep
this mechanism simple for now, but in the future a more advanced peer discovery
mechanism may be utilized.

### Verifier

Most of the lite client logic resides in the verifier, where commits for a
header are actually verified. In order to abstract over all the data in the
Tendermint data structures, we use a set of traits that include only the
information absolutely necessary for the lite client. From this perspective, we
have the following traits.

### Core Traits

A Header has only a height, time, and validator sets, and can be identified by its hash:

```rust
pub trait Header {
    fn height(&self) -> Height;
    fn bft_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    fn hash(&self) -> Hash;
}
```

A validator contributes a positive voting power if a message was correctly signed by it,
otherwise it contributes 0. We could represent this with a single methods that
returns either 0 or the voting power, but it's probably clearer with two methods:

```rust
pub trait Validator {
    fn power(&self) -> u64;
    fn verify_signature(&self, sign_bytes: Bytes, signature: Bytes) -> bool;
}
```

A vote corresponds to the message signed by a particular validator for a particular commit.
Since it is expected to be for a particular commit, it does not need to include
that information explicitly (eg. the height, the round, the block ID, etc.).
Instead, from the verifie's perspective, all that information can be contained implicitly within the message
bytes that are being signed. When those message bytes and the signature are passed to 
`validator.verify_signature` for the corresponding validator, it should return true:

```rust
pub trait Vote {
    fn validator_id(&self) -> ValID;
    fn sign_bytes(&self) -> Bytes;
    fn signature(&self) -> Bytes;
}
```

A validator set is a collection of validators that we will want to iterate over.
We also need to know its hash, so we can check it against what's in a header. 
We use an associated type for the validator:

```rust
pub trait ValidatorSet {
    type Validator: Validator;

    fn hash(&self) -> Hash;
    fn into_vec(&self) -> Vec<Self::Validator>;
}
```

It may seem more natural to use an `iter` method here than `into_vec`, since
ultimately we only need to iterate over the underlying associated type.
However, this turned out to be much more difficult than expected due to the lack
of size information in the `Iter` trait. It's possible to use another associated
type, eg. `type ValidatorIter: ExactSizeIterator<Item = Self::Validator>;`, but it just
complicates things significantly. If this becomes a performance issue (ie. we
end up having to copy the vector of validators), we can address it then.

Finally, a commit is a collection of votes that we will want to iterate over.
We also need to check the hash of the header the commit is for. Since the
commit is for a particular header, we can require all votes to be for that header, 
and otherwise ignore them. Again, we use an associated type:

```rust
pub trait Commit {
    type Vote: Vote;

    fn header_hash(&self) -> Hash;
    fn into_vec(&self) -> Vec<Option<Self::Vote>>;
}
```

Note the `Option` here. When it is `None`, it indicates that either:

- there was no vote from this validator
- the validator voted nil
- the validator noted for some other block

We may want to be more strict here in distinguishing between these cases. On the
one hand, upcoming changes to the commit structure will prevent votes from being included if they are from the
wrong block (see https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-025-commit.md).
On the other, we may want the validator to verify the votes for nil - even
though they don't contribute anything to the voting power, they serve as an
extra validity check. I propose for now that we use the simple Option time and
ignore the distinction between these cases, but that we revisit at a future
date. 

Note also that in Tendermint, commits are for a particular block ID, which
includes both a header hash and a "parts set hash". The latter is completely
irelevant to the light client, and can only be verified by downloading the full
block. Hence it is effectively ignored here. It would be great if Tendermint
could disentangle commits to the proposal block parts for gossip (ie. the parts
set hash) from commits to the header itself (ie. the header hash), but that's
left for the future.

### Implementing Traits

The tendermint-rs library includes Header, Vote, Validator, ValidatorSet, and
Commit data types. However, rather than use these types directly, the lite
client library is written more abstractly to use traits that contain only the
necessary information from these more concrete types. While this may turn out to
be an unecessarily eager abstraction (as we do not forsee alternative
implementations of these traits in the short term), it does provide a very clear
depiction of what is required for lite client verification, and surfaces certain
design issues in the underlying Tendermint blockchain (eg. the `BlockID` issue
referenced above).

## Verification

Since we know the validators for a commit (ie. the number of validators should match the length of our votes vector), 
we can iterate over them, check the signatures, and sum the voting power.

An error should be returned if:

- any signature is invalid 
- 2/3 or less of the voting power signed

So we can have a function like:

```rust
fn verify_commit_full<V, C>(vals: V, commit: C) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
{
```

### "Trusting" Verification

To skip (ie. the "trusting method"), we have to check if +1/3 of validators at some past height signed the commit, 
before we can check if +2/3 of the validators for the current height signed.
To do this, we have to know which validator a vote is from and be able to look them up in the validator
set. Hence we need to extend our ValidatorSet trait to permit such lookups:

```rust
pub trait ValidatorSetLookup: ValidatorSet {
    fn validator(&self, val_id: ValID) -> Option<Self::Validator>;
}
```

Now for each vote in the commit, we can check if the validator existed in our
trusted validator set, and thus if +1/3 of the trusted validators signed the new
commit. We can use a function with the same signature:

```rust
fn verify_commit_trusting<V, C>(vals: V, commit: C) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
{
```

### Validation

Most of the above is about checking the signatures and voting power in commits, but we also need to perform other validation checks, 
like that the validator set hashes match what are in the header, and the lite
client's trusted state actually hasn't expired.

TODO - more

    
## Status

Proposed

## Consequences

### Positive

TODO

### Negative

TODO

### Neutral

TODO
