# Draft of Light Client Supervisor for discussion

## TODOs:

- move part I of verification here (or copy)

- incorporate the structure of Stevan's Rust supervisor design
   - new versions of `verifytotarget` and `backwards` that take as
     input a single lightblock and return a fully verified lightstore
   - update tags to ".2"
   - lightstore.update: remove Unverified upon leaving verifyTotarget
 
check that all is addressed:

- https://github.com/informalsystems/tendermint-rs/issues/499
- https://github.com/informalsystems/tendermint-rs/pull/509
- https://github.com/tendermint/spec/issues/131
- https://github.com/informalsystems/tendermint-rs/issues/461


- links to verification and detection specs



# Light Client Sequential Supervisor

The light client implements a read operation of a
[header][TMBC-HEADER-link] from the [blockchain][TMBC-SEQ-link], by
communicating with full nodes, a so-called primary and several
so-called witnesses. As some full nodes may be faulty, this
functionality must be implemented in a fault-tolerant way.

In the Tendermint blockchain, the validator set may change with every
new block.  The staking and unbonding mechanism induces a [security
model][TMBC-FM-2THIRDS-link]: starting at time *Time* of the
[header][TMBC-HEADER-link],
more than two-thirds of the next validators of a new block are correct
for the duration of *TrustedPeriod*. 

[Light Client Verification](TODO) implements the fault-tolerant read
operation designed for this security model. That is, it is safe if the
model assumptions are satisfied and makes progress if it communicates
to a correct primary.

However, if the [security model][TMBC-FM-2THIRDS-link] is violated,
faulty peers (that have been validators at some point in the past) may
launch attacks on the Tendermint network, and on the light
client. These attacks as well as an axiomatization of blocks in
general are defined in [a document that contains the definitions that
are currently in detection.md](TODO). 

If there is a light client attack (but no
successful attack on the network), the safety of the verification step
may be violated (as we operate outside its basic assumption).
The light client also
contains a defense mechanism against light clients attacks, called detection.

[Light Client Detection](TODO) implements a cross check of the result
of the verification step. If there is a light client attack, and the
light client is connected to a correct peer, the light client as a
whole is safe, that is, it will not operate on invalid
blocks. However, in this case it cannot successfully read, as
inconsistent blocks are in the system. However, in this case the
detection performs a distributed computation that results in so-called
evidence. Evidence can be used to prove
to a correct full node that there has been a
light client attack.

[Light Client Evidence Accountability](TODO) is a protocol run on a
full node to check whether submitted evidence indeed proves the
existence of a light client attack. Further, from the evidence and its
own knowledge about the blockchain, the full node computes a set of
bonded full nodes (that at some point had more than one third of the
voting power) that participated in the attack that will be reported
via ABCI to the application.

In this document we specify

- Initialization of the Light Client
- The interaction of [verification](TODO) and [detection](TODO)

The details of these two protocols are captured in their own
documents, as is the [accountability](TODO) protocol.

> Another related line is IBC attack detection and submission at the
> relayer, as well as attack verification at the IBC handler. This
> will call for yet another spec.


# Status

This document is work in progress. In order to develop the
specification step-by-step,
it assumes certain details of [verification](TODO) and
[detection](TODO) that are not specified in the respective current
versions yet. This inconsistencies will be addresses over several
upcoming PRs.


# Part I - Tendermint Blockchain

TODO

# Part II - Sequential Problem Definition 


#### **[LC-SEQ-INIT-LIVE.1]**: 
Upon initialization, the light client gets as input a header of the
blockchain, or the genesis file of the blockchain, and eventually
stores a header of the blockchain.

> TODO: be more precise about heights in spec above

#### **[LC-SEQ-LIVE.1]**: 
The light client gets a sequence of heights as inputs. For each input
height *targetHeight*, it eventually stores the header of height
*targetHeight*.

#### **[LC-SEQ-SAFE.1]**:

The light client never stores a header which is not in the blockchain.

# Part III - Light Client as Distributed System

## Computational Model

TODO: primary, witness (should be mainly discussed in detection
spec).

TODO: always connected to a correct peer (should be mainly discussed
in detection spec). responds in time

TODO: no main chain fork, that is, we assume all correct peers agree
on one blockchain. But TFM is violated, that is, there are bonded
validators that execute a light client attack
(details should be mainly discussed in
verification and detection specs). 

## Distributed Problem Statement

### Two Kinds of Liveness

In case of light client attacks, the sequential problem statement
cannot always be satisfied. The lightclient cannot decide which block
is from the chain and which is not. As a result, the light client just
creates evidence, submits it, and terminates. 
For the liveness property, we thus add the
possibility that instead of adding a lightblock, we also might terminate
in case there is an attack.


#### **[LC-DIST-TERM.1]**:

The light client either runs forever or it *terminates on attack*.

### Design choices

#### [LC-DIST-STORE.1]:
The light client has a local data structure called LightStore 
that contains light blocks (that contain a header). 

TODO: put light store functions here (some are still in verification.md)

#### **[LC-DIST-SAFE.1]**:
It is always the case that every header in *LightStore* was 
generated by an instance of Tendermint consensus.

#### **[LC-DIST-LIVE.1]**:

Whenever the light client gets a new height *h* as input,
- and there is
no light client attack up to height *h*, then the lightclient
eventually puts the lightblock of height *h* in the lightstore and
wait for another input.
- otherwise, that is, if there
is a light client attack on height *h*, then the light client
must perform one of the following:
    - it terminates on attack.
    - it eventually puts the lightblock of height *h* in the lightstore and
wait for another input.

### Solving the sequential specification

TODO: as one of the peers is correct, the lightclient will always
download a correct lightblock (by the assumption that there is no fork
on the chain and all correct peers agree on the blokchain). -> safety

For liveness there are three scenarios
1. no light client attack: all lightblocks match, and they are written
   to the lightstore
2. there is an attack
    a. one of the peers sends a lightblock involved in the attack to
    the lightclient -> by assumption that there is a correct peer, it
    will be detected, and we terminate on attack
	b. the lightclient does not learn a faulty lightblock -> similar
    to case 1.

# Part IV - Light Client Supervisor Protocol


We provide a specification for a sequential Light Client Supervisor.
The local code for verification is presented by a sequential function
`Sequential-Supervisor` to highlight the control flow of this
functionality. Each lightblock is first verified with a primary, and then
cross-checked with secondaries, and if all goes well, the lightblock
is
added to the
lightstore. That is, no lightblock is ever removed from the
lightstore, and all lightblocks in the lightstore are trusted. 

> We note that if a different concurrency model is considered
> for an implementation, the semantics of the lightstore might change:
> In a concurrent implementation, we might do verification for some
> height *h*, add the
> lightblock to the lightstore, and start concurrent threads that
> - do verification for the next height *h' != h*
> - do cross-checking for height *h*. If we find an attack, we remove
>   *h* from the lightstore.
>
> Thus, this concurrency model changes the semantics of the
> lightstore (not all lightblocks are trusted; they may be removed if
> we find a problem). Whether this is desirable, and whether the gain in
> performance is worth it, we keep for future versions/discussion of
> lightclient protocols.


## Definitions

### Data Types

The core data structure of the protocol is the LightBlock.

#### **[LC-DATA-LIGHTBLOCK.1]**:
```go
type LightBlock struct {
                Header          Header
                Commit          Commit
                Validators      ValidatorSet
                NextValidators  ValidatorSet
                Provider        PeerID
}
```     

#### **[LC-DATA-LIGHTSTORE.1]**:

LightBlocks are stored in a structure which stores all LightBlock from
initialization or received from peers.

```go
type LightStore struct {
        ...
}

```

The LightStore exposes the following functions to query stored LightBlocks.

TODO: copy used new functions with tags.



### Inputs

The lightclient is initialized with LCInitData

#### **[LC-DATA-INIT.1]**:
```go
type LCInitData struct {
	lightBlock     LightBlock
	genesisDoc     GenesisDoc
}
```

where only one of the components must be provided. `GenesisDoc` is
defined in the [Tendermint
Types](https://github.com/tendermint/tendermint/blob/master/types/genesis.go). 

#### **[LC-DATA-GENESIS.1]**:

```go
type GenesisDoc struct {
	GenesisTime     time.Time                `json:"genesis_time"`
	ChainID         string                   `json:"chain_id"`
	InitialHeight   int64                    `json:"initial_height"`
	ConsensusParams *tmproto.ConsensusParams `json:"consensus_params,omitempty"`
	Validators      []GenesisValidator       `json:"validators,omitempty"`
	AppHash         tmbytes.HexBytes         `json:"app_hash"`
	AppState        json.RawMessage          `json:"app_state,omitempty"`
}
```

We use the following function
`makeblock` so that we create a lightblock from the genesis
file in order to do verification based on the data from the genesis
file using the same verification function we use in normal operation.

#### **[LC-FUNC-MAKEBLOCK.1]**:
```go
func makeblock (genesisDoc GenesisDoc) (lightBlock LightBlock))
```
- Implementation remark
    - none
- Expected precondition
    - none
- Expected postcondition
    - lightBlock.Header.Height =  genesisDoc.InitialHeight
    - lightBlock.Header.Time = genesisDoc.GenesisTime
    - lightBlock.Header.LastBlockID = nil
    - lightBlock.Header.LastCommit = nil
    - lightBlock.Header.Validators = genesisDoc.Validators
    - lightBlock.Header.NextValidators = genesisDoc.Validators
    - lightBlock.Header.Data = nil 
    - lightBlock.Header.AppState =  genesisDoc.AppState
    - lightBlock.Header.LastResult = nil
	- lightBlock.Commit = nil
	- lightBlock.Validators = genesisDoc.Validators
	- lightBlock.NextValidators = genesisDoc.Validators
	- lightBlock.Provider = nil
- Error condition
    - none
----


### Configuration Parameters

**TODO:** trusting period

### Variables

**TODO**

- PeerList. 

It exposes the functions

- primary()
- secondaries()
- replace_primary()
- replace secondary(peerID PeerID)

### Assumptions

init data is OK

### Invariants

#### **[LC-DATA-PEERLIST.1]:**
The peer list contains a primary and a secondary.

> If the invariant is violated, the light client does not have enough
> peers to download headers from. As a result, the light client
> needs to terminate in case this invariant is violated. 



## Supervisor

### Outline

The supervisor implements the functionality of the lightclient. It is
initialized with a genesis file or with a lightblock the user
trusts. This initialization is subjective, that is, the security of
the lightclient is based on the validity of the input. If the genesis
file or the lightblock deviate from the actual ones on the blockchain,
the lightclient provides no guarantees.

After initialization, the supervisor awaits an input, that is, the
height of the next lightblock that should be obtained. Then it
downloads, verifies, and cross-checks a lightblock, and if all tests
go through, the light block (and possibly other lightblocks) are added
to the lightstore, which is returned in an output event to the user.

The following main loop does the interaction with the user (input,
output) and calls the following two functions:
- `InitLightClient`: it initializes the lightstore either with the
  provided lightblock or with the lightblock that corresponds to the
  first block generated by the blockchain (by the validators defined
  by the genesis file)
- `VerifyAndDetect`: takes as input a lightstore and a height and
  returns the updated lightstore. 

#### **[LC-FUNC-SUPERVISOR.1]:**

```go
func Sequential-Supervisor (initdata LCInitData) (Error) {
							
	lightStore,result := InitLightClient(initData);
	if result != OK {
	    return result;
	}
	
    loop {
	    // get the next height
        nextHeight := input();
		
		lightStore,result := VerifyAndDetect(lightStore, nextHeight);
		
		if result == OK {
		    output(LightStore)
		}
		else {
		    return result
		}
		// QUESTION: is it OK to generate output event in normal case,
	    // and terminate with failure in the (light client) attack case?
	}
}
```
- Implementation remark
    - infinite loop unless a light client attack is detected
- Expected precondition
    - *LCInitData* contains a genesis file or a lightblock. 
- Expected postcondition
    - if a light client attack is detected: it stops and submits
      evidence (in `InitLightClient` or `VerifyAndDetect`)
    - otherwise: non. It runs forever.
- Invariant: *lightStore* contains trusted lightblocks only.
- Error condition
    - if `InitLightClient` or `VerifyAndDetect` fails (if a attack is
	detected, or if [LCV-INV-TP.1] is violated)
----


### Details of the Functions

#### Initialization

The light client is based on subjective initialization. It has to
trust the initial data given to it by the user. It cannot do any
detection of attack. So either upon initialization we obtain a
lightblock and just initialize the lightstore with it. Or in case of a
genesis file, we download, verify, and cross-check the first block, to
initialize the lightstore with this first block. The reason is that
we want to maintain [LCV-INV-TP.1] from the beginning.

> If the lightclient is initialized with a lightblock, one might think
> it may increase trust, when one cross-checks the initial light
> block. However, if a peer provides a conflicting
> lightblock, the question is to distinguish the case of a
> [bogus](TODO) block (upon which operation should proceed) from a
> [light client attack](TODO) (upon which operation should stop). In
> case of a bogus block, the lightclient might be forced to do
> backwards verification until the blocks are out of the trusting
> period, to make sure no previous validator set could have generated
> the bogus block, which effectively opens up a DoS attack on the lightclient
> without adding effective robustness.

#### **[LC-FUNC-INIT.1]:**
```go
func InitLightClient (initData LCInitData) (LightStore, Error) {

    if LCInitData.LightBlock != nil {
	    // we trust the provided initial block. 
        newblock := LCInitData.LightBlock
    }
    else {
	    genesisBlock := makeblock(initData.genesisDoc);

        result := NoResult;
        while result != ResultSuccess {
            current = FetchLightBlock(PeerList.primary(), genesisBlock.Header.Height + 1)
            // QUESTION: is the height with "+1" OK?
			
            if CANNOT_VERIFY = ValidAndVerify(genesisBlock, current) {
			// TODO: remove "trusted.Commit is a commit for the header
		    // trusted.Header, i.e., it contains the correct hash of the
		    // header, and +2/3 of signatures" from validAndVerified precondition
		        peerList.replacePrimary();
			}
			else {
			    result = ResultSuccess
			}
		}
		
        // cross-check
		Evidences := AttackDetector(genesisBlock, current)
		// TODO: wrap current in an auxiliary lightstore
        if Evidences.Empty {
		    newBlock := current
	    }
		else {
		    submitEvidence(Evidences);
            return(nil, ErrorAttack);
		}
    }

    lightStore := new LightStore;
    lightStore.Add(newBlock); //TODO: add Add to lightstore functions
    return (lightStore, OK);
}
							
```
- Implementation remark
    - none
- Expected precondition
	- *LCInitData* contains either a genesis file of a lightblock 
	- if genesis it passes `ValidateAndComplete()` see [Tendermint](TODO)
- Expected postcondition
    - *lightStore* initialized with trusted lightblock. It has either been
      cross-checked (from genesis) or it has initial trust from the
      user.
- Error condition
    - if precondition is violated
	- empty peerList
----


#### Main verification and detection logic



#### **[LC-FUNC-MAIN-VERIF-DETECT.1]:**

```go
func VerifyAndDetect (lightStore LightStore, targetHeight Height) 
                     (LightStore, Result) {

    b1, r1 = lightStore.Get(targetHeight)
    if r1 = true {
        // block already there
        return (lightStore, ResultSuccess)
    }

    // get the lightblock with maximum height smaller than targetHeight
	// would typically be the heighest, if we always move forward
    root_of_trust, r2 = lightStore.LatestPrevious(targetHeight);
    if r2 = false {
	    // there is no lightblock from which we can do forward
		// (skipping) verification. Thus we have to go backwards.
		// No cross-check needed. We trust hashes. Therefore, we
	    // directly return the result
    return Backwards(peerList.primary(), lightStore.lowest, targetHeight)
	    // TODO: in Backwards definition pointers need to be fixed to
		//       predecessor
		// TODO: add tag pointer to Backwards
		// TODO: define lightStore.lowest
	}
	else {
        // Forward verification + detection
        result := NoResult;
        while result != ResultSuccess {
            verifiedLS,result := VerifyToTarget(peerList.primary(),
			                                    root_of_trust, 
												nextHeight);
			// TODO: in verifytotarget return only verification chain
		    // TODO: add tag pointer to verifytotarget
            if result == ResultFailure {				
				// pick new primary (promote a secondary to primary)
	            peerList.Replace_Primary();
			}
        }
		
		// Cross-check
		// TODO: fix parameters and functions
        Evidences := AttackDetector(root_of_trust, verifiedLS);
		// TODO: add tag pointer to AttackDetector
		if Evidences.Empty {
		    // no attack detected, we trust the new lightblock
            lightStore.store_chain(verifidLS);
			// TODO: add store_chain function
			return (lightStore, OK);
        } 
        else {
		    // there is an attack, we exit
            return(lightStore, ErrorAttack);
        }
	}
}

```
- Implementation remark
    - none
- Expected precondition
	- none
- Expected postcondition
    - lightblock of height *targetHeight* (and possibly additional blocks) added to *lightStore* 
- Error condition
    - an attack is detected
	- [LC-DATA-PEERLIST-INV.1] is violated
----



TODO: submitEvidence(Evidences); in detector spec


### Solving the distributed specification

TODO



# Part V:  Discussions on Changes Semantics of the LightStore

In a previous version (TODO: versioning, link),
a lightblock in the lightstore can be in one of the
following states:
- StateUnverified
- StateVerified
- StateFailed
- StateTrusted

The intuition is that `StateVerified` captures that the lightblock has
been verified with the primary, and `StateTrusted` is the state after
successful cross-checking with the secondaries.

Assuming there is **always one correct node among primary and
secondaries**, and there is no fork on the blockchain, lightblocks that
are in `StateTrusted` can be used by the user with the guarantee of
"finality". If a block in  `StateVerified` is used, it might be that
detection later finds a fork, and a roll-back might be needed.

**Remark:** The assumption of one correct node, does not render
verification useless. It is true that if the primary and the
secondaries return the same block we may trust it. However, if there
is a node that provides a different block, the light node still needs
verification to understand whether there is a fork, or whether the
different block is just bogus (without any support of some previous
validator set).

**Remark:** A light node may choose the full nodes it communicates
with (the light node and the full node might even belong to the same
stakeholder) so the assumption might be justified in some cases.

In the future, we will do the following changes
   - we assume that only from time to time, the light node is
     connected to a correct full node
   - this means for some limited time, the light node might have no
     means to defend against light client attacks
   - as a result we do not have finality
   - once the light node reconnects with a correct full node, it
     should detect the light client attack and submit evidence.

Under these assumptions, `StateTrusted` loses its meaning. As a
result, it should be removed from the API. We suggest that we replace
it with a flag "trusted" that can be used 
- internally for efficiency reasons (to maintain
  [LCD-INV-TRUSTED-AGREED.1] until a fork is detected)
- by light client based on the "one correct full node" assumption


----




