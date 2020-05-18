# Core Verification

The light client implements a read operation of a
[header][TMBC-HEADER-link] from the [blockchain][TMBC-SEQ-link], by
communicating with full nodes.  As some full nodes may be faulty, this
functionality must be implemented in a fault-tolerant way.

In the Tendermint blockchain, the validator set may change with every
new block.  The staking and unbonding mechanism induces a [security
model][TMBC-FM-2THIRDS-link]: starting at time *Time* of the
[header][TMBC-HEADER-link],
more than two-thirds of the next validators of a new block are correct
for the duration of *TrustedPeriod*. The fault-tolerant read
operation is designed for this security model.

The challenge addressed here is that the light client might have a
block of height *h1* and needs to read the block of height *h2*
greater than *h1*.  Checking all headers of heights from *h1* to *h2*
might be too costly (e.g., in terms of energy for mobile devices).
This specification tries to reduce the number of intermediate blocks
that need to be checked, by exploiting the guarantees provided by the
[security model][TMBC-FM-2THIRDS-link].

# Outline

- [Part I](#part-i---tendermint-blockchain): Introduction of
 relevant terms of the Tendermint
blockchain.

- [Part II](#part-ii---sequential-definition-of-the-verification-problem): Introduction
of the problem addressed by the Lightclient Verification protocol.
    - [Verification Informal Problem
      statement](#Verification-Informal-Problem-statement): For the general
      audience, that is, engineers who want to get an overview over what
      the component is doing from a bird's eye view.
    - [Sequential Problem statement](#Sequential-Problem-statement):
      Provides a mathematical definition of the problem statement in
      its sequential form, that is, ignoring the distributed aspect of
      the implementation of the blockchain.

- [Part III](#part-iii---light-client-as-distributed-system): Distributed
  aspects of the light client, system assumptions and temporal
  logic specifications.

  - [Incentives](#incentives): how faulty full nodes may benefit from
    misbehaving and how correct full nodes benefit from cooperating.
  
  - [Computational Model](#Computational-Model):
      timing and correctness assumptions.

  - [Distributed Problem Statement](#Distributed-Problem-Statement):
      temporal properties that formalize safety and liveness
      properties in the distributed setting.

- [Part IV](#part-iv---light-client-verification-protocol):
  Specification of the protocols.

     - [Definitions](#Definitions): Describes inputs, outputs,
       variables used by the protocol, auxiliary functions

     - [Core Verification](#core-verification): gives an outline of the solution,
       and details of the functions used (with preconditions,
       postconditions, error conditions).

     - [Liveness Scenarios](#liveness-scenarios): when the light
       client makes progress depends heavily on the changes in the
       validator sets of the blockchain. We discuss some typical scenarios.

In this document we quite extensively use tags in order to be able to
reference assumptions, invariants, etc. in future communication. In
these tags we frequently use the following short forms:

- TMBC: Tendermint blockchain
- SEQ: for sequential specifications
- LCV: Lightclient Verification
- LIVE: liveness
- SAFE: safety
- INV: invariant
- A: assumption



# Part I - Tendermint Blockchain


## Header Fields necessary for the Light Client

#### **[TMBC-HEADER]**:
A set of blockchain transactions is stored in a data structure called
*block*, which contains a field called *header*. (The data structure
*block* is defined [here][block]).  As the header contains hashes to
the relevant fields of the block, for the purpose of this
specification, we will assume that the blockchain is a list of
headers, rather than a list of blocks. 

#### **[TMBC-HASH-UNIQUENESS]**:
We assume that every hash in the header identifies the data it hashes. 
Therefore, in this specification, we do not distinguish between hashes and the 
data they represent.


#### **[TMBC-HEADER-FIELDS]**:
A header contains the following fields:

 - `Height`: non-negative integer
 - `Time`: time (integer)
 - `LastBlockID`: Hashvalue
 - `LastCommit` DomainCommit
 - `Validators`: DomainVal
 - `NextValidators`: DomainVal
 - `Data`: DomainTX
 - `AppState`: DomainApp
 - `LastResults`: DomainRes


#### **[TMBC-SEQ]**:

The Tendermint blockchain is a list *chain* of headers. 


#### **[TMBC-VALIDATOR-PAIR]**:

Given a full node, a 
*validator pair* is a pair *(peerID, voting_power)*, where 
  - *peerID* is the PeerID (public key) of a full node, 
  - *voting_power* is an integer (representing the full node's
  voting power in a certain consensus instance).
  
> In the Golang implementation the data type for *validator
pair* is called `Validator`


#### **[TMBC-VALIDATOR-SET]**:

A *validator set* is a set of validator pairs. For a validator set
*vs*, we write *TotalVotingPower(vs)* for the sum of the voting powers
of its validator pairs.

#### **[TMBC-VOTE]**:
A *vote* contains a `prevote` or `precommit` message sent and signed by
a validator node during the execution of [consensus][arXiv]. Each 
message contains the following fields
   - `Type`: prevote or precommit
   - `Height`: positive integer
   - `Round` a positive integer
   - `BlockID` a Hashvalue of a block (not necessarily a block of the chain)



#### **[TMBC-COMMIT]**:
A commit is a set of `precommit` message.


## Tendermint Failure Model

#### **[TMBC-AUTH-BYZ]**:
We assume the authenticated Byzantine fault model in which no node (faulty or
correct) may break digital signatures, but otherwise, no additional
assumption is made about the internal behavior of faulty 
nodes. That is, faulty nodes are only limited in that they cannot forge
messages.


#### **[TMBC-TIME-PARAMS]**:
A Tendermint blockchain has the following configuration parameters:
 - *unbondingPeriod*: a time duration.
 - *trustingPeriod*: a time duration smaller than *unbondingPeriod*.


#### **[TMBC-CORRECT]**:
We define a predicate *correctUntil(n, t)*, where *n* is a node and *t* is a 
time point. 
The predicate *correctUntil(n, t)* is true if and only if the node *n* 
follows all the protocols (at least) until time *t*.



#### **[TMBC-FM-2THIRDS]**:
If a block *h* is in the chain,
then there exists a subset *CorrV*
of *h.NextValidators*, such that:
  - *TotalVotingPower(CorrV) > 2/3
    TotalVotingPower(h.NextValidators)*; cf. [TMBC-VALIDATOR-SET]
  - For every validator pair *(n,p)* in *CorrV*, it holds *correctUntil(n,
    h.Time + trustingPeriod)*; cf. [TMBC-CORRECT]


> The definition of correct
> [**[TMBC-CORRECT]**](TMBC-CORRECT-link) refers to realtime, while it
> is used here with *Time* and *trustingPeriod*, which are "hardware
> times".  We do not make a distinction here.

## What the Light Client Checks


> From [TMBC-FM-2THIRDS] we directly derive the following observation:

#### **[TMBC-VAL-CONTAINS-CORR]**:

Given a (trusted) block *tb* of the blockchain, a given set of full nodes 
*N* contains a correct node at a real-time *t*, if
   - *t - trustingPeriod < tb.Time < t*
   - the voting power in tb.NextValidators of nodes in *N* is more
     than 1/3 of *TotalVotingPower(tb.NextValidators)*

> The following describes how a commit for a given block *b* must look
> like.

#### **[TMBC-SOUND-DISTR-POSS-COMMIT]**:
For a block *b*, each element *pc* of *PossibleCommit(b)* satisfies:
  - each vote *v* in *pc* satisfies
     * *pc* contains only votes (cf. [TMBC-VOTE])
	 by validators from *b.Validators*
     * v.blockID = hash(b)
	 * v.Height = b.Height  
	 **TODO:** complete the checks here
  - the sum of the voting powers in *pc* is greater than 2/3
  *TotalVotingPower(b.Validators)*

> The following property comes from the validity of the [consensus][arXiv]: A
> correct validator node only sends `prevote` or `precommit`, if
> `BlockID` of the new (to-be-decided) block is equal to the hash of
> the last block.

#### **[TMBC-VAL-COMMIT]**:

If for a block *b*,  a commit *c*
  - contains at least one validator pair *(v,p)* such that *v* is a 
    **correct** validator node, and
  - is contained in *PossibleCommit(b)*
  
then the block *b* is on the blockchain.




## Context of this document



In this document we specify the light client verification component,
called *Core Verification*.  The *Core Verification* communicates with
a full node.  As full nodes may be faulty, it cannot trust the
received information, but the light client has to check whether the
header it receives coincides with the one generated by Tendermint
consensus.

The two 
 properties [[TMBC-VAL-CONTAINS-CORR]][TMBC-VAL-CONTAINS-CORR-link] and
[[TMBC-VAL-COMMIT]][TMBC-VAL-COMMIT-link]  formalize the checks done
 by this specification:
Given a trusted block *tb* and an untrusted block *ub* with a commit *cub*,
one has to check that *cub* is in *PossibleCommit(ub)*, and that *cub*
contains a correct node using *tb*.



# Part II - Sequential Definition of the Verification Problem


## Verification Informal Problem statement


Given a height *targetHeight* as an input, the *Verifier* eventually
stores a header *h* of height *targetHeight* locally.  This header *h*
is generated by the Tendermint [blockchain][blockchain]. In
particular, a header that was not generated by the blockchain should
never be stored.


## Sequential Problem statement

#### **[LCV-SEQ-LIFE]**: 
The *Verifier* gets as input a height *targetHeight*, and eventually stores the
header of height *targetHeight* of the blockchain.

#### **[LCV-SEQ-SAFE]**:
The *Verifier* never stores a header which is not in the blockchain.


# Part III - Light Client as Distributed System

## Incentives

Faulty full nodes may benefit from lying to the light client, by making the
light client accept a block that deviates (e.g., contains additional 
transactions) from the one generated by Tendermint consensus. 
Users using the light client might be harmed by accepting a forged header.

The [fork detector][failuredetector] of the light client may help the
correct full nodes to understand whether their header is a good one.
Hence, in combination with the light client detector, the correct full
nodes have the incentive to respond.  We can thus base liveness
arguments on the assumption that correct full nodes reliably talk to
the light client.



## Computational Model

#### **[LCV-A-PEER]**:
The verifier communicates with a full node called *primary*. No assumption is made about the full node (it may be correct or faulty).

#### **[LCV-A-COMM]**:
Communication between the light client and a correct full node is
reliable and bounded in time. Reliable communication means that
messages are not lost, not duplicated, and eventually delivered. There
is a (known) end-to-end delay *Delta*, such that if a message is sent
at time *t* then it is received and processes by time *t + Delta*.
This implies that we need a timeout of at least *2 Delta* for remote
procedure calls to ensure that the response of a correct peer arrives
before the timeout expires.

#### **[LCV-A-TFM]**:
The Tendermint blockchain satisfies the Tendermint failure model [**[TMBC-FM-2THIRDS]**][TMBC-FM-2THIRDS-link].

#### **[LCV-A-VAL]**:
The system satisfies [**[TMBC-AUTH-BYZ]**][TMBC-Auth-Byz-link] and
[**[TMBC-FM-2THIRDS]**][TMBC-FM-2THIRDS-link]. Thus, there is a
blockchain that satisfies the soundness requirements (that is, the
validation rules in [[block]]).



## Distributed Problem Statement

### Two Kinds of Termination

We do not assume that *primary* is correct. Under this assumption no
protocol can guarantee the combination of the sequential
properties. Thus, in the (unreliable) distributed setting, we consider
two kinds of termination (successful and failure) and we will specify
below under what (favorable) conditions *Core Verification* ensures to
terminate successfully, and satisfy the requirements of the sequential
problem statement:


#### **[LCV-DIST-TERM]**:
*Core Verification* either *terminates
successfully* or it *terminates with failure*.


### Design choices

#### **[LCV-DIST-STORE]**:
*Core Verification* has a local data structure called *LightStore* that
contains light blocks (that contain a header). For each light block we
record whether it is verified.


#### **[LCV-DIST-PRIMARY]**:
*Core Verification* has a local variable *primary* that contains the PeerID of a full node.

#### **[LCV-DIST-INIT]**:
*LightStore* is initialized with a header *trustedHeader* that was correctly 
generated by the Tendermint consensus. We say *trustedHeader* is verified.

### Temporal Properties

#### **[LCV-DIST-SAFE]**:
It is always the case that every verified header in *LightStore* was 
generated by an instance of Tendermint consensus.

#### **[LCV-DIST-LIFE]**:
From time to time, a new instance of *Core Verification* is called with a
height *targetHeight*. Each instance must eventually terminate. 
  - If
     - the  *primary* is correct, and 
     - *LightStore* always contains a verified header whose age is less than the
        trusting period,  
    then *Core Verification* adds a verified header *hd* with height
    *targetHeight* to *LightStore* and it **terminates successfully**
	

> These definitions imply that if the primary is faulty, a header may or
> may not be added to *LightStore*. In any case,
> [**[LCV-DIST-SAFE]**](#lcv-vc-inv) must hold.

> The invariant [**[LCV-DIST-SAFE]**](#lcv-dist-safe) and the liveness 
> requirement [**[LCV-DIST-LIFE]**](#lcv-dist-life)
> allow that verified headers are added to *LightStore* whose 
> height was not passed
> to the verifier (e.g., intermediate headers used in bisection; see below).

> Note that for liveness, initially having a *trustedHeader* within
> the *trustinPeriod* is not sufficient. However, as this
> specification will leave some freedom with respect to the strategy
> in which order to download intermediate headers, we do not give a
> more precise liveness specification here. After giving the
> specification of the protocol, we will discuss some liveness
> scenarios [below](#liveness-scenarios).

### Solving the sequential specification

This specification provides a partial solution to the sequential specification.
The *Verifier* solves the invariant of the sequential part

[**[LCV-DIST-SAFE]**](#lcv-vc-inv) => [**[LCV-SEQ-SAFE]**](#lcv-seq-inv)

In the case the primary is correct, and there is a recent header in *LightStore*, the verifier satisfies the liveness requirements.

⋀ *primary is correct*  
⋀ always ∃ verified header in LightStore. *header.Time* > *now* - *trustingPeriod*  
⋀ [**[LCV-A-Comm]**](#lcv-a-comm) ⋀ (
       ( [**[TMBC-CorrFull]**][TMBC-CorrFull-link] ⋀
         [**[LCV-DIST-LIVE]**](#lcv-vc-live) )
       ⟹ [**[LCV-SEQ-LIVE]**](#lcv-seq-live)
)


# Part IV - Light Client Verification Protocol

We provide a specification for Light Client Verification. The local
code for verification is presented by a sequential function
`VerifyToTarget` to highlight the control flow of this functionality.
We note that if a different concurrency model is considered for
an implementation, the sequential flow of the function may be
implemented with mutexes, etc. However, the light client verification
is partitioned into three blocks that can be implemented and tested
independently:

- `FetchLightBlock` is called to download a light block (header) of a
  given height from a peer.
- `ValidAndVerified` is a local code that checks the header.
- `Pivot` decides which height to try to verify next. We keep this
  underspecified as different implementations (currently in Goland and
  Rust) may implement different optimizations here. We just provide
  necessary conditions on how the height may evolve.
  

> `ValidAndVerified` is the function that is sometimes called "Light
> Client" in the IBC context.


## Definitions

### Data Types

The core data structure of the protocol is the LightBlock.

```go
type LightBlock struct {
		Header          Header
		Commit          Commit
		Validators      ValidatorSet
		NextValidators  ValidatorSet
		Provider        PeerID
}
```	

LightBlocks are stored in a structure which stores all LightBlock from
initialization or received from peers.

```go
type LightStore struct {
	...
}

```

Each LightBlock is in one of the following states:
```go
type VerifiedState int

const (
	StateUnverified = iota + 1
	StateVerified
	StateFailed
)
```

The LightStore exposes the following functions to query stored LightBlocks.

```go
func (ls LightStore) Get(height Height) (LightBlock, bool) 
```
- Expected postcondition
  - returns a LightBlock at a given height or false in the second argument if
    the LightStore does not contain the specified LightBlock.


```go
func (ls LightStore) LatestVerified() LightBlock
```
- Expected postcondition
   - returns the heighest verified light block:`


```go
func (ls LightStore) Update(lightBlock LightBlock, verfiedState VerifiedState)
```
- Expected postcondition
   - The state of the LightBlock is set to *verifiedState*.


### Inputs
- *lightStore*: stores light blocks that have been downloaded and that
    passed verification. Initially it contains a light block with
	*trustedHeader*.
- *primary*: peerID
- *targetHeight*: they height of the needed header


### Configuration Parameters

- *trustThreshold*: a float. Can be used if correctness should not be based on more voting power and 1/3.
- *trustingPeriod*: a time duration [**[TMBC-TIME_PARAMS]**][TMBC-TIME_PARAMS-link].
- *clockDrift*: a time duration. Correction parameter dealing with only approximately synchronized clocks.


### Variables


- *nextHeight*: initially *targetHeight*
  > *nextHeight* should be thought of the "height of the next header we need
  > to download and verify"



### Assumptions

#### **[LCV-A-INIT]**:
- *trustedHeader* is from the blockchain

- *targetHeight > LightStore.LatestVerified.Header.Height*

### Invariants

#### **[LCV-INV-TP]**:
It is always the case that *LightStore.LatestVerified.Header.Time > now - trustingPeriod*.

> If the invariant is violated, the light client does not have a
> header it can trust. A trusted header must be obtained externally, 
> its trust can only be based on social consensus.


### Messages

**TODO:** 
 
### Remote Functions
  ```go
func FetchLightBlock(peer PeerID, height Height) LightBlock
```
- Implementation remark
   - RPC to peer at *PeerID*
   - Request message: **TODO**
   - Response message: **TODO**
- Expected precodnition
  - `height` is less than or equal to height of the peer
- Expected postcondition
  - if *node* is correct: 
    - Returns the LightBlock *lb* of height `height`
      that is consistent with the blockchain
	- *lb.provider = peer* 
    - *lb.Header* is a header consistent with
      the blockchain
    - *lb.Validators* is the validator set of the
      blockchain at height *nextheight*
    - *lb.NextValidators* is the validator set of the
      blockchain at height *nextheight + 1*
  - if *node* is faulty: Returns a LightBlock with arbitrary content 
    [**[TMBC-AUTH-BYZ]**][TMBC-Auth-Byz-link]
- Error condition
   * if *n* is correct: precondition violated **TODO:** mention message
   * if *n* is faulty: arbitrary error
   * if *lb.provider != peer* 
---



## Core Verification

### Outline

The `VerifyToTarget` is the main function and uses the following functions.
- `FetchLightBlock` is called to download the next light block. It is
  the only function that communicates with other nodes
- `ValidAndVerified` checks whether header is valid and checks if a
  new lightBlock should be trusted
  based on a previously verified lightBlock.
- `Pivot` decides which height to try to verify next

In the following description of `VerifyToTarget` we do not deal with error
handling. If any of the above function returns an error, VerifyToTarget just
passes the error on.

```go
func VerifyToTarget(primary PeerID, lightStore LightStore,
                    targetHeight Height) (LightStore, Result) {

    nextHeight := targetHeight

    for lightStore.LatestVerified.height < targetHeight {
    
        // Get next LightBlock for verification
        current, found := lightStore.Get(nextHeight)
        if !found {
            current = FetchLightBlock(primary, nextHeight)
            lightStore.Update(current, StateUnverified)
        }

        // Verify
        verdict = ValidAndVerified(lightStore.LatestVerified, current)
        
        // Decide whether/how to continue
        if verdict == OK {
            lightStore.Update(current, StateVerified)
        }
        else if verdict == CANNOT_VERIFY {
            // do nothing
			// the light block current passed validation, but the validator
            // set is too different to verify it. We keep the state of
			// current at StateUnverified. For a later iteration, Pivot
			// might decide to try verification of that light block again.
        }    
        else { 
            // verdict is some error code
            lightStore.Update(current, StateFailed)
            // possibly remove all LightBlocks from primary
            return (lightStore, ResultFailure)
        } 
        nextHeight = Pivot(lightStore, nextHeight, targetHeight)
    }
    return (lightStore, ResultSuccess)
}
```

- Expected precondition
   - *lightStore* contains a LightBlock within the *trustingPeriod*
   - *targetHeight* is greater than the height of all the LightBlocks
     in *lightStore*
- Expected postcondition: 
   - returns *lightStore* that contains a LightBlock that corresponds 
     to a block
     of the blockchain of height *targetHeight* (that is, the
     LightBlock has been added to *lightStore*)
- Error conditions
   - if the precondition is violated
   - if `ValidAndVerified` or `FetchLightBlock` report an error
   - if [**[LCV-INV-TP]**](#LCV-INV-TP) is violated
  


### Details of the Functions




```go
func ValidAndVerified(trusted LightBlock, untrusted LightBlock) Result
```
- **TODO:** check and make complete
- Expected precondition
  - none
- Expected precondition:
   - *untrusted* is valid, that is, satisfies the soundness [checks][block]
   - *untrusted* is **well-formed**, that is,
        - *untrusted.Header.Time < now + clockDrift*
        - *untrusted.Validators = hash(untrusted.Header.Validators)*
        - *untrusted.NextValidators = hash(untrusted.Header.NextValidators)*
   - *trusted.Header.Time > now - trustingPeriod*
   - *trusted.Commit* is a commit is for the header 
     *trusted.Header*, i.e. it contains
     the correct hash of the header
   - the `Height` and `Time` of `trusted` are smaller than the Height and 
  `Time` of `untrusted`, respectively
   - the *untrusted.Header* is well-formed (passes the tests from
     [[block]]), and in particular
      - if the untrusted header `unstrusted.Header` is the immediate 
	  successor  of  `trusted.Header`, then it holds that
	      - *trusted.Header.NextValidators = 
		  untrusted.Header.Validators*, and
		  moreover, 
		  - *untrusted.Header.Commit* 
		     - contains signatures by more than two-thirds of the validators 
		     - contains no signature from nodes that are not in *trusted.Header.NextValidators*
- Expected postcondition: 
    - Returns `OK`:
        - if *untrusted* is the immediate successor of *trusted*,
          or otherwise,
        - if
		   - signatures of a set of validators that have more than
             *max(1/3,trustThreshold)* of voting power in
             *trusted.Nextvalidators* is contained in
             *untrusted.Commit* (that is, header passes the tests
             [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link]
             and [**[TMBC-VAL-COMMIT]**][TMBC-VAL-COMMIT-link])
    - Returns `CANNOT_VERIFY` if:
         - *untrusted* is *not* the immediate successor of
           *trusted*
		   and the  *max(1/3,trustThreshold)* threshold is not reached
           (that is, if
	     [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link] 
	     fails and header is does not violate the soundness
         checks [[block]]).
- Error condition: 
   - if precondition violated 
   - If *trusted.Header.Time > now - trustingPeriod* the blabla
---



```go
func Pivot(lightStore, nextHeight, targetHeight) Height
```
- Implementation remark: If picks the next height to be verified. 
  We keep the precise choice of the next header under-specified. It is
  subject to performance optimizations that do not influence the correctness
- Expected postcondition: return *H* s.t.
   1. if *lightStore.LatestVerified.Height = nextHeight* and
      *lightStore.LatestVerified < targetHeight* then  
	  *nextHeight < H <= targetHeight*
   2. if *lightStore.LatestVerified.Height < nextHeight* and
      *lightStore.LatestVerified.Height < targetHeight* then  
	  *lightStore.LatestVerified.Height < H < nextHeight*
   3. if *lightStore.LatestVerified.Height = targetHeight* then  
     *H =  targetHeight*

> Case i. captures the case where the light block at height
> *nextHeight* has been verified, and we can choose a height closer to
> the *targetHeight*. As we
> get the *lightStore* as parameter, the choice of the next height can
> depend on the *lightStore*, e.g., we can pick a height for which we
> have already downloaded a light block. 
> In Case iii. is a special case when we have
> verified the *targetHeight*. In Case ii. the header of *nextHeight*
> could not be verified, and we need to pick a smaller height. 


### Solving the distributed specification

*trustedStore* is implemented by the light blocks in lightStore that
have the state *StateVerified*.

**TODO: check**
#### Argument for [**[LCV-DIST-SAFE]**](#lcv-dist-safe):

- `ValidAndVerified` implements the soundness checks and the checks 
  [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link] and 
  [**[TMBC-VAL-COMMIT]**][TMBC-VAL-COMMIT-link] under
  the assumption [**[TMBC-FM-2THIRDS]**][TMBC-FM-2THIRDS-link]
- Only if `ValidAndVerified` returns with `OK`, the state of a light block is
  set to *StateVerified*.


#### Argument for [**[LCV-DIST-LIFE]**](#lcv-dist-life):

- If *primary* is correct, 
    - `FetchLightBlock` will always return a light block consistent
      with the blockchain
    - `ValidAndVerified` either verify the header using the trusting
      period or falls back to sequential
      verification
    - If [**[LCV-INV-TP]**](#LCV-INV-TP) holds, eventually every
	  header will be verified and core verification **terminates successfully**.
    - successful termination depends on the age of *lightStore.LatestVerified*
      (for instance, initially on the age of  *trustedHeader*) and the
      changes of the validator sets on the blockchain.
	  We will give some examples [below](#liveness-scenarios).
- If *primary* is faulty,
    - it either provides headers that pass all the tests, and we
      return with the header 
	- it provides one header that fails a test, core verification
      **terminates with failure**.



## Liveness Scenarios

The liveness argument above assumes [**[LCV-INV-TP]**](#LCV-INV-TP)
which requires that there is a header that does not expire before the
target height is reached. Here we discuss scenarios to ensure this.

Let *startHeader* be *LightStore.LatestVerified* when core
verification is called (*trustedHeader*) and *startTime* be the time
core verification is invoked.

In order to ensure liveness, *LightStore* always needs to contain a
verified (or initially trusted) header whose time is within the
trusting period. To ensure this, core verification needs to add new
headers to *LightStore* and verify them, before all headers in
*LightStore* expire.

#### Many changes in validator set

 Let's consider `Pivot` implements
 bisection, that is, it halves the distance.
 Assume the case where the validator set changes completely in each
block. Then the 
 method in this specification needs to
sequentially verify all headers. That is, for

- *W = log_2 (targetHeight - startHeader.Height)*,

*W* headers need to be downloaded and checked before the
header of height *startHeader.Height + 1* is added to *LightStore*.

- Let *Comp*
  be the local computation time needed to check headers and signatures
  for one header.
- Then we need in the worst case *Comp + 2 Delta* to download and
  check one header.
- Then the first time a verified header could be added to *LightStore* is
  startTime + W * (Comp + 2 Delta)
- [TP] However, it can only be added if we still have a header in
  *LightStore*, 
  which is not
  expired, that is only the case if
    - startHeader.Time > startTime + WCG * (Comp + 2 Delta) -
      trustingPeriod, 
	- that is, if core verification is started at  
	  startTime < startHeader.Time + trustingPeriod -  WCG * (Comp + 2 Delta) 

- one may then do an inductive argument from this point on, depending
  on the implementation of `Pivot`. We may have to account for the 
  headers that are already
  downloaded, but they are checked against the new *LightStore.LatestVerified*.

> We observe that
> the worst case time it needs to verify the header of height
> *targetHeight* depends mainly on how frequent the validator set on the
> blockchain changes. That core verification terminates successfully
> crucially depends on the check [TP], that is, that the headers in
> *LightStore* do not expire in the time needed to download more
> headers, which depends on the creation time of the headers in
> *LightStore*. That is, termination of core verification is highly
> depending on the data stored in the blockchain.


> The current light client core verification protocol exploits that, in
> practice, changes in the validator set are rare. For instance,
> consider the following scenario.




#### No change in validator set

If on the blockchain the validator set of the block at height
*targetHeight* is equal to *startHeader.NextValidators*:
- there is one round trip in `FetchLightBlock` to download the light
 block
 of height
  *targetHeight*, and *Comp* to check it.
- as the validator sets are equal, `Verify` returns `OK`, if
  *startHeader.Time > now - trustingPeriod*.
- that is, if *startTime < startHeader.Header.Time + trustingPeriod -
  2 Delta - Comp*, then core verification terminates successfully










# References

[[block]] Specification of the block data structure. 

[[blockchain]] The specification of the Tendermint blockchain. Tags refering to this specification are labeled [TMBC-*].

[[failuredetector]] The specification of the light client fork detector.

[[fullnode]] Specification of the full node API

[[lightclient]] The light client ADR [77d2651 on Dec 27, 2019].


[block]: https://github.com/tendermint/spec/blob/master/spec/blockchain/blockchain.md
[blockchain]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md
[TMBC-HEADER-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-header
[TMBC-SEQ-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-seq
[TMBC-CorrFull-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-corrfull
[TMBC-Auth-Byz-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-auth-byz
[TMBC-Sign-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-sign
[TMBC-FaultyFull-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-faultyfull
[TMBC-TIME_PARAMS-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-time_params
[TMBC-FM-2THIRDS-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-fm-2thirds
[TMBC-VAL-CONTAINS-CORR-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-val-contains-corr
[TMBC-VAL-COMMIT-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-val-commit
[TMBC-SOUND-DISTR-LAST-COMMIT-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-sound-distr-last-commit
[TMBC-SOUND-DISTR-POSS-COMMIT-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-sound-distr-posscommit


[TMBC-INV-SIGN-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-inv-sign
[TMBC-INV-VALID-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-inv-valid

[LCV-VC-LIVE-link]: https://github.com/informalsystems/VDD/tree/master/lightclient/verification.md#lcv-vc-live

[lightclient]: https://github.com/interchainio/tendermint-rs/blob/e2cb9aca0b95430fca2eac154edddc9588038982/docs/architecture/adr-002-lite-client.md
[failuredetector]: https://github.com/informalsystems/VDD/blob/master/liteclient/failuredetector.md
[fullnode]: https://github.com/tendermint/spec/blob/master/spec/blockchain/fullnode.md

[FN-LuckyCase-link]: https://github.com/tendermint/spec/blob/master/spec/blockchain/fullnode.md#fn-luckycase

[blockchain-validator-set]: https://github.com/tendermint/spec/blob/master/spec/blockchain/blockchain.md#data-structures
[fullnode-data-structures]: https://github.com/tendermint/spec/blob/master/spec/blockchain/fullnode.md#data-structures

[FN-ManifestFaulty-link]: https://github.com/tendermint/spec/blob/master/spec/blockchain/fullnode.md#fn-manifestfaulty

[arXiv]: https://arxiv.org/abs/1807.04938
