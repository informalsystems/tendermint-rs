

# TODO:

- light fork -> light client attack
- primary / witness

## context of this document

### Light node

- initialization

- `verification.md` describes the verifier and gives specification in
  case of TFM
  
- if TFM is violated, safety may be violated. That is, the primary
  convinces us of a faulty block
  
- `detection.md`:
   - we do cross checking with secondaries
   - if we have a situation
       - we compute a minimal evidence. 
	   - the assumption is that one
         peer is correct, and willing to cooperate
	   - by cooperating we will generate minimal evidence  
	     (it seems that cooperation is less fragile to DOS than
         submitting unlimited data via traces)
   - submit evidence [*]
   
### related components

- IBC fork detection and submission at the relayer

- evidence handling / attack isolation at a full node - the receiving
  end of [*]

## structure of this document?

   - move part I of verification here (or copy)
   - sequential statement: copy from verification
   - distributed statement: 
       - if TFM holds
	   - if TFM might be violated
 

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


- put computation and submission if "minimal" PoF into a function that
  hides floating details
  
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


# Status

This document is work in progress. In order to develop the
specification step-by-step,
it assumes certain details of [verification](TODO) and
[detection](TODO) that are not specified in the respective current
versions yet. This inconsistencies will be addresses over several
upcoming PRs.


# Outline

TODO

# Part I - Tendermint Blockchain

TODO

# Part II - Sequential Problem Definition 



#### **[LC-SEQ-INIT-LIVE.1]**: 
Upon initialization, the light client gets as input a header of the
blockchain, or the genesis file of the blockchain, and eventually
stores a header of the blockchain.

#### **[LC-SEQ-LIVE.1]**: 
The light client gets a sequence of heights as inputs. For each input
height *targetHeight*, it eventually stores the header of height
*targetHeight* of the blockchain.

#### **[LC-SEQ-SAFE.1]**:

The light client never stores a header which is not in the blockchain.

# Part III - Light Client as Distributed System

## Computational Model

TODO: primary, witness from detection

TODO: always connected to a correct peer

TODO: no main chain attack, that is, we assume all correct peers have
knowledge of the blockchain

## Distributed Problem Statement

### Two Kinds of Liveness

TODO: light client attack or no light client attack

#### **[LCV-DIST-TERM.1]**:

*Core Verification* either runs forever or it *terminates on attack*.

### Design choices

#### [LC-DIST-STORE.1]:
The light client has a local data structure called LightStore 
that contains light blocks (that contain a header). 


#### [LCV-DIST-PRIMARY.1]:
The light client
has a local variable primary that contains the PeerID of a full node.

TODO: secondaries?

#### **[LC-DIST-SAFE.1]**:
It is always the case that every header in *LightStore* was 
generated by an instance of Tendermint consensus.

#### **[LCV-DIST-LIVE.1]**:

From time to time, a new instance of *Core Verification* is called with a
height *targetHeight* greater than the height of any header in *LightStore*. 
Each instance must eventually terminate. 
  - If
     - the  *primary* is correct (and locally has the block of
       *targetHeight*), and 
     - *LightStore* always contains a verified header whose age is less than the
        trusting period,  
    then *Core Verification* adds a verified header *hd* with height
    *targetHeight* to *LightStore* and it **terminates successfully**
 


### Design choices

https://github.com/tendermint/tendermint/blob/master/types/genesis.go

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

#### **[LC-FUNC-INIT.1]:**
```go
func InitLightClient (initData LCInitData) (LightStore, Error) {

    if LCInitData.LightBlock != nil {
	    // TODO: make rational section on init
	    // we trust the provided initial block. No cross checking 
		// is necessary, can increase the trust. It would only open up
		// a possibility for DOS attacks
        newblock := LCInitData.LightBlock
    }
    else {
	    genesisBlock := makeblock(initData.genesisDoc);
		// TODO: add section on rationale
		// We want to populate the lightstore with a complete
		// lightblock (no empty lastblockid, etc.)
		// TODO: 1 or 2?
	    current = FetchLightBlock(primary, 2)
        
		// https://github.com/tendermint/spec/blob/8dd2ed4c6fe12459edeb9b783bdaaaeb590ec15c/spec/core/data_structures.md
		// how the initial verification works is not so clear from the spec
        // TODO: remove "trusted.Commit is a commit for the header
		// trusted.Header, i.e., it contains the correct hash of the
		// header, and +2/3 of signatures" from validAndVerified
        precondition
        if CANNOT_VERIFY = ValidAndVerify(genesisBlock, current) {
		   // genesis bad or primary faulty
		   // TODO: retry within a loop
		}
		
		
        // cross-check
		Evidences := Forkdetector(genesisBlock, b2)
        if Evidences.Empty {
		    newBlock := block
	    }
		else {
		    submitEvidence(Evidences);
            return(nil, ErrorFork);
		}
    }

    lightStore := new LightStore;
    lightStore.Update(newBlock);
    return (lightStore, OK);
}
							
```
**TODO:** finish conditions
- Implementation remark
- Expected precondition
	- *LCInitData* contains either a genesis file of a lightblock 
	- if genesis it passes `ValidateAndComplete()`
- Expected postcondition
    - *lightStore* initialized with trusted lightblock. It has either been
      cross checked (from genesis) or it has initial trust from the
      user.
	  
- Error condition
    - if precondition is violated
----

#### **[LC-FUNC-MAIN-VERIF-DETECT.1]:**

```go
func VerifyAndDetect (primary PeerID, 
                        lightStore LightStore, 
						targetHeight Height) (LightStore, Result) {

    b1, r1 = lightStore.Get(targetHeight)
    if r1 = true {
        // block already there
        return (lightStore, ResultSuccess)
    }

    // get the lightblock with maximumheight smaller than targetHeight
	// would typically be the heighest, if we always move forward
    root_of_trust, r2 = lightStore.LatestPrevious(targetHeight);
    if r2 = false {
		// Backwards verification. No cross-check needed. We trust hashes.		
	    return Backwards(primary, lightStore.lowest, targetHeight)
	    // TODO: in Backwards definition pointers need to be fixed to
		//       predecessor
	}
	else {
        // Forward verification + detection
        result := NoResult;
        while result != ResultSuccess {

            verifiedLS,result := VerifyToTarget(primary,
			                                    root_of_trust, 
												nextHeight);
			// TODO: in verifytotarget return only verification chain
            if result == ResultFailure {				
				// pick new primary (promote a secondary to primary)
				/// and delete all lightblocks above
	            // LastTrusted (they have not been cross-checked)
	            Replace_Primary();
			}
        }
		
		// Cross-check
		// TODO: fix parameters and functions
        Evidences := Forkdetector(root_of_trust, verifiedLS);
        if Evidences.Empty {
		    // no fork detected with secondaries, we trust the new
			// lightblock
            lightStore.store_chain(verifidLS);
			return (lightStore, OK);
        } 
        else {
		    // there is a fork, we submit the proofs and exit
            submitEvidence(Evidences);
            return(lightStore, ErrorFork);
        }
	}
}

```

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
		
		lightStore,result := VerifyAndDetect(primary, lightStore, nextHeight);
		
		if result == OK {
		    output(LightStore)
		}
		else {
		    return result
		}
		// QUESTION: is it OK to generate output event in normal case,
	    // and terminate with failure in fork case?
	}
}
```
**TODO:** finish conditions  
**TODO:** lightStore invariant, with pointers to previous block  
**TODO:** only verified lightblocks in lightStore  
- Implementation remark
- Expected precondition
    - *lightStore* initialized with trusted header
	- *PoFs* empty
- Expected postcondition
    - runs forever, or
	- is terminated by user and satisfies LightStore invariant, or **TODO**
	- has submitted proof of fork upon detecting a fork
- Error condition
    - if `InitLightClient` fails
----

# Semantics of the LightStore

Currently, a lightblock in the lightstore can be in one of the
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




