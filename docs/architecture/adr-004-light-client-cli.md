# ADR 004: Light Client CLI

## Changelog

2020-01-22: Some content copied from old ADR-002

## Status

WIP. Just copied over from old ADR. Needs rework

## Context


### State

The light node state contains the following:

- current height (H) - height for the next header we want to verify
- last header (H-1) - the last header we verified
- current validators (H) - validators for the height we want to verify (including all validator pubkeys and voting powers)

It also includes some configuration, which contains:

- trusting period
- initial list of full nodes
- method (sequential or skipping)
- trust level (if method==skipping)

The node is initialized with a trusted header for some height H-1
(call this header[H-1]), and a validator set for height H (call this vals[H]).

The node may be initialized by the user with only a height and header hash, and
proceed to request the full header and validator set from a full node. This
reduces the initialization burden on the user, and simplifies passing this
information into the process, but for the state to be properly initialized it
will need to get the correct header and validator set before starting the light
client syncing protocol.

The configuration contains an initial list of full nodes (peers).
For the sake of simplicity, one of the peers is selected as the "primary", while the
rest are considered "backups". Most of the data is downloaded from the primary,
and double checked against the backups.

The state is considered "expired" if the difference between the current time and
the time from the trusted header is greater than a configurable "trusting
period". If at any point the state is expired, the node should log an error and
exit - it's needs to be manually reset.

### Syncer

The Syncing co-ordinates the syncing and is the highest level component. 
We consider two approaches to syncing the light node: sequential and skipping.

#### Sequential Sync

Inital state: 

    - time T
    - height H 
    - header[H-1]
    - vals[H]

Here we describe the happy path:

1) Request header[H], commit[H], and vals[H+1] from the primary, and check that they are well formed and from the correct height
2) Pass header[H], commit[H], vals[H], and vals[H+1] to the verification library, which will:

    - check that vals[H] and vals[H+1] are correctly reflected in header[H]
    - check that commit[H] is for header[H]
    - check that +2/3 of the validators correctly signed the hash of header[H]

3) Request header[H] from each of the backups and check that they match header[H] received from the primary
4) Update the state with header[H] and vals[H+1], and increment H
5) return to (1)

If (1) or (2) fails, mark the primary as bad and select a new peer to be the
primary.

If (3) returns a conflicting header, verify the header by requesting the
corresponding commit and running the verification of (2). If the verification
passes, there is a fork, and evidence should be published so the validators get
slashed. We leave the mechanics of evidence to a future document. For now, the
light client will just log an error and exit. If the verification fails, it
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
to get the latest height of the full node (for skipping verification). It 
uses the following trait (see below for definitions of the referenced types):

```rust
pub trait Requester {
    type SignedHeader: SignedHeader;
    type ValidatorSet: ValidatorSet;

    fn signed_header<H>(&self, h: H) -> Result<Self::SignedHeader, Error>
        where H: Into<Height>;

    fn validator_set<H>(&self, h: H) -> Result<Self::ValidatorSet, Error>
        where H: Into<Height>;
}
```

Note that trait uses `Into<Height>` which is a common idiom for the codebase.

