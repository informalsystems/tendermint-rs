# ADR 005: Light Client Fork Detection

## Changelog

2020-01-22: Some content copied from old ADR-002

## Status

WIP. Just copied over from old ADR. Needs rework

## Context

In addition to the core verification logic, the light node needs a way to
receive data from full nodes, to detect conflicting information, and to report
on conflicts. While there are many ways for a full node to provide bad
information, what we're really looking for is misbehaviour by the validators,
which is reflected in conflicting commits (ie. commits for different blocks at
the same height). 

### Detect

The detection module is for checking if any of the backup nodes
are reporting conflicting information. It requests headers from each backup node
and compares them with a verified header from the primary. If there is a
conflict, it attempts to verify the conflicting header via the verifier. If it
can be verified, it indicates an attack on the light clients that should be
punishable. The relevant information (ie. the two conflicting commits) are
passed to the publisher.

### Publisher

For now, the publisher just logs an error, writes the conflicting commits to a
file, and exits. We leave it to a future document to describe how this
information can actually be published to the blockchain so the validators can be
punished. Tendermint may need to expose a new RPC endpoint to facilitate this.

See related [Tendermint issue #3244](https://github.com/tendermint/tendermint/issues/3244).

### Address Book

For now this is a simple list of HTTPS addresses corresponding to full nodes
that the node connects to. One is randomly selected to be the primary, while
others serve as backups. It's essential that the light node connect to at least
one correct full node in order to detect conflicts in a timely fashion. We keep
this mechanism simple for now, but in the future a more advanced peer discovery
mechanism may be utilized.

