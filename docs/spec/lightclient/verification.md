**Preparation for high-level English spec for new architecture** 

# Core Verification


The light client implements a read operation of a
[header][TMBC-HEADER-link] from the [blockchain][TMBC-SEQ-link], by
communicating with full nodes.  As some full nodes may be faulty, this
functionality must be implemented in a fault-tolerant way.

For the purpose of this specification, we assume that the blockchain
 is a list of headers, rather than a list of blocks, by
 [**[TMBC-HEADER]**][TMBC-HEADER-link].

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

**TODO:** Similar to Fastsync spec

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
*validator pair* is a pair *(address, voting_power)*, where 
  - *address* is the address (public key) of a full node, 
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
message contain the following fields
   - `Type`: prevote or precommit
   - `Height`: positive integer
   - `Round` a positive integer
   - `BlockID` a Hashvalue of a block (not necessarily a block of the chain)



#### **[TMBC-COMMIT]**:
A commit is a set of votes.

**TODO:** clarify whether `prevote` or `precommit` are equivalent in
the Commit.

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
  - contains at least one validator pair *(v,p)* such that *v* is a correct
    validator node, and
  - is contained in *PossibleCommit(b)*
  
then the block *b* is on the blockchain.




## Context of this document



In this document we specify the light client verification component,
called *Core Verification*.  The *Core Verification* communicates with
a full node.  As full nodes may be faulty, it cannot trust the
received information, but the light client has to check whether the
header it receives coincides with the one generated by Tendermint
consensus.

 To do verification (in particular the function
`Verify`) checks based on these properties, the two 
 properties [[TMBC-VAL-CONTAINS-CORR]][TMBC-VAL-CONTAINS-CORR-link] and
[[TMBC-VAL-COMMIT]][TMBC-VAL-COMMIT-link]  formalize the checks done
 by this specification:
Given a trusted block *tb* and an untrusted block *ub* with a commit *cub*,
one has to check that *cub* is in *PossibleCommit(ub)*, and that *cub*
contains a correct node using *tb*.



# Part II - Sequential Definition of the Light Client Problem


## Informal Problem statement


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
*Core Verification* has a local data structure called *trustedStore* that
contains headers.


#### **[LCV-DIST-PRIMARY]**:
*Core Verification* has a local variable *primary* that contains the Address (ID) of a full node.

#### **[LCV-DIST-INIT]**:
*trustedStore* is initialized with a header *trustedHeader* that was correctly 
generated by the Tendermint consensus.

### Temporal Properties

#### **[LCV-DIST-SAFE]**:
It is always the case that every header in *trustedStore* was generated by an instance of Tendermint consensus.

#### **[LCV-DIST-LIFE]**:
From time to time, a new instance of *Core Verification* is called with a
height *targetHeight*. Each instance must eventually terminate. 
  - If
     - the  *primary* is
       correct, and 
     - *trustedStore* contains a header whose age is less than the
        trusting period,  
	   **TODO:** be more precise. Having a header in the beginning is
        not sufficient. Having a header upon termination might allow
        trivial solutions.

    then *Core Verification* adds a header *hd* with height
    *targetHeight* to *trustedStore* and it **terminates successfully**
	



> These definitions imply that if the primary is faulty, a header may or
> may not be added to *trustedStore*. In any case,
> [**[LCV-DIST-SAFE]**](#lcv-vc-inv) must hold.

> The invariant [**[LCV-DIST-SAFE]**](#lcv-dist-safe) and the liveness 
> requirement [**[LCV-DIST-LIFE]**](#lcv-dist-life)
> allow that headers are added to *trustedStore* whose height was not passed
> to the verifier (e.g., intermediate headers used in bisection; see below).



### Solving the sequential specification

This specification provides a partial solution to the sequential specification.
The *Verifier* solves the invariant of the sequential part

[**[LCV-DIST-SAFE]**](#lcv-vc-inv) => [**[LCV-SEQ-SAFE]**](#lcv-seq-inv)

In the case the primary is correct, and there is a recent header in *State*, the verifier satisfies the liveness requirements.

/\ "correct primary"  
/\ always \E header in trustedStore. header.Time > now - *trustingPeriod*  
/\ [**[LCV-A-Comm]**](#lcv-a-comm) /\
       [**[TMBC-CorrFull]**][TMBC-CorrFull-link] /\
       [**[LCV-DIST-LIVE]**](#lcv-vc-live)  
       => [**[LCV-SEQ-LIVE]**](#lcv-seq-live)


# Part IV - Light Client Verification Protocol

**TODO:** intro paragraph

## Definitions

### Data Types

The core data structure of the protocol is the LightBlock.
```go
type LightBlock struct {
	Header          Header
	Commit          Commit
	Validators      ValidatorSet
	NextValidators  ValidatorSet
	Provider        Address
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
    passed verification. Initially it contains *trustedLightBlock*.
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

- *headerToVerify*: a light block. Initially nil


### Auxiliary Functions

#### **[LCV-FUNC-REF]**:
- *refHeader*: is the header from *trustedStore* with the maximal
  height
  
- *currentHeight*: *refHeader.Header.Height*


### Assumptions

#### **[LCV-A-INIT]**:
- *trustedHeader* is from the blockchain

- *targetHeight > currentHeight*

### Invariants

#### **[LCV-INV-TP]**:
It is always the case that *refHeader.Header.Time > now - trustingPeriod*.

> If the invariant is violated, the light client does not have a
> header it can trust. A trusted header must be obtained externally, 
> its trust can only be based on social consensus.


### Messages

**TODO:** 
 
### Remote Functions
  ```go
func FetchLightBlock(node PeerID, height Height) LightBlock
```
- Implementation remark
   - RPC to peer at *PeerID*
   - Request message: **TODO**
   - Response message: **TODO**
- Expected precodnition
  - `height` is less than or equal to height of the peer
- Expected postcondition
  - if *node* is correct: Returns the LightBlock of height `height`
  that is consistent with the blockchain
  - if *node* is faulty: Returns a LightBlock with arbitrary content
- Error condition
   * if *n* is correct: precondition violated **TODO:** mention message
   * if *n* is faulty: arbitrary error

---


## Core Verification

### Outline

The `VerifyToTarget` is the main function and uses the following functions.
- `FetchLightBlock` is called to download the next light block
- `ValidLightBlock` checks whether header is valid. 
- `SufficientVotingPower` checks if a new lightBlock should be trusted
  based on a previously verified lightBlock.

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
        }    
        else { 
            // verdict == INVALID 
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
   - *lightStore* contains a LightBlock that corresponds to a block
     of the blockchain of height *targetHeight* (that is, the
     LightBlock has been added to *lightStore*)
- Error conditions
   - if the precondition is violated
   - if `ValidLightBlock` or `FetchLightBlock` report an error
   - if [**[LCV-INV-TP]**](#LCV-INV-TP) is violated
  

### Solving the distributed specification

#### Argument for [**[LCV-DIST-SAFE]**](#lcv-dist-safe):

- `IO` will always return a well-formed header *h* or report an error
    - if it is well-formed it will be put into *headerToVerify*
	- on error, `demuxer` returns with error, and nothing is added to
      *trustedStore*, which trivially would ensure safety
- `Verify` implements the checks 
  [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link] and 
  [**[TMBC-VAL-COMMIT]**][TMBC-VAL-COMMIT-link] under
  the assumption [**[TMBC-FM-2THIRDS]**][TMBC-FM-2THIRDS-link]
- Only if `Verify` returns with `OK`, `Scheduler` adds
  *headerToVerify* to *trustedStore*, that is, only if it was
  generated by the blockchain 

#### Argument for [**[LCV-DIST-LIFE]**](#lcv-dist-life):

- If *primary* is correct, 
    - `IO` will always return the header from the blockchain
    - `Verify` either verify the header or fall back to sequential
      verification
    - If [**[LCV-INV-TP]**](#LCV-INV-TP) holds, eventually every
	  header will be verified and core verification **terminates successfully**.
    - successful termination depends on the age of the *refHeader*
      (for instance, initially on the age of  *trustedHeader*) and the
      changes of the validator sets on the blockchain.
	  We will give some examples [below](#liveness-scenarios).
- If *primary* is faulty,
    - it either provides headers that pass all the tests, and we
      return with the header 
	- it provides one header that fails a test, core verification
      **terminates with failure**.


### Details of the Functions


```go
func FetchLightBlock(peer Address, height Height) LightBlock
```
-  Implementation remark
   - Used to communicate with a full node *n* at address *addr* via 
     RPCs `Commit` and `Validators` 
   - The only function that makes external calls!
   - This function make externals RPC calls to the full node; 
      - `Validators(addr, ht)`
	  - `Validators(addr, ht + 1)` 
	  - `Commit (addr, ht)`
- Expected precondition
  - none
- Expected postcondition: 
  -. It returns a light block *lb* 
  - *lb.Provider = addr*
  - *lb* is **well-formed**, that is,
     - *lb.Header.Time < now + clockDrift*
     - *lb.Validators = hash(lb.Header.Validators)*
     - *lb.NextValidators = hash(lb.Header.NextValidators)*
  - If *n* is correct:
    - *lb.Header* is a header consistent with
      the blockchain
    - *lb.Validators* is the validator set of the
      blockchain at height *nextheight*
    - *lb.NextValidators* is the validator set of the
      blockchain at height *nextheight + 1*
  - If *n* is faulty, *lb.Header* and *lb.Validators* and
    *lb.NextValidators* are arbitrary  [**[TMBC-AUTH-BYZ]**][TMBC-Auth-Byz-link]
- Error conditions
  - precondition violated
  - downloaded information does not allow to form a well-formed light block.
  - If *n* is faulty

---


```go
func sufficientVotingPower(tursted LightBlock, untrusted LightBlock) bool
```
TODO

```go
func validLightBlock(lightBlock LightBlock) bool 
```
TODO

- Expected precondition:
   - *trustedLB.Header.Time > now - trustingPeriod*
   - *trustedLB.Commit* is a commit is for the header 
     *trustedLB.Header*, i.e. it contains
     the correct hash of the header
   - the `Height` and `Time` of `trustedLB` are smaller than the Height and 
  `Time` of `untrustedLB`, respectively
   - the *untrustedLB.Header* is well-formed (passes the tests from
     [[block]]), and in particular
      - if the untrusted header `unstrustedLB.Header` is the immediate 
	  successor  of  `trustedLB.Header`, then it holds that
	      - *trustedLB.Header.NextValidators = 
		  untrustedLB.Header.Validators*, and
		  moreover, 
		  - *untrustedLB.Header.Commit* 
		     - contains signatures by more than two-thirds of the validators 
		     - contains no signature from nodes that are not in *trustedLB.Header.NextValidators*
- Expected postcondition: 
    - Returns `OK`:
        - if *untrustedLB* is the immediate successor of *trustedLB*,
          or otherwise,
        - if
		   - signatures of a set of validators that have more than
             *max(1/3,trustThreshold)* of voting power in
             *trustedLB.Nextvalidators* is contained in
             *untrustedLB.Commit* (that is, header passes the tests
             [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link]
             and [**[TMBC-VAL-COMMIT]**][TMBC-VAL-COMMIT-link])
    - Returns `CANNOT_VERIFY` if:
         - *untrustedLB* is *not* the immediate successor of
           *trustedLB*
		   and the  *max(1/3,trustThreshold)* threshold is not reached
           (that is, if
	     [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link] 
	     fails and header is does not violate the soundness
         checks [[block]]).
- Error condition: 
   - if precondition violated 
   - If *trustedLB.Header.Time > now - trustingPeriod* the blabla
---




### Liveness Scenarios

Let *startHeader* be *refHeader* when core verification is called
(*trustedHeader*) and *startTime* be the time core verification is
invoked.

In order to ensure liveness, *trustedStore* always needs to contain a
header whose time is within the trusting period. To ensure this, core
verification needs to add new headers to *trustedStore*, before all
headers in *trustedStore* expire.

#### Many changes in validator set

Assume the case where the validator set changes completely in each
block. Then the bisection method in this specification needs to
sequentially all headers. That is, for

- *W = log_2 (targetHeight - startHeader.Height)*,

*W* headers need to be downloaded and checked before the
header of height *startHeader.Height + 1* is added to *trustedStore*.

- Let *Comp*
  be the local computation time needed to check headers and signatures
  for one header.
- Then we need in the worst case *Comp + 2 Delta* to download and
  check one header.
- Then the first time a header could be added to *trustedStore* is
  startTime + W * (Comp + 2 Delta)
- [TP] However, it can only be added if we still have a header in
  *trustedStore*, 
  which is not
  expired, that is only the case if
    - startHeader.Time > startTime + WCG * (Comp + 2 Delta) -
      trustingPeriod, 
	- that is, if core verification is started at  
	  startTime < startHeader.Time + trustingPeriod -  WCG * (Comp + 2 Delta) 

- one may then do an inductive argument from this point on. (To be
  precise we have to account for the headers that are already
  downloaded, but they are checked against the new *refHeader*)).

> We observe that
> the worst case time it needs to verify the header of height
> *targetHeight* depends mainly on how frequent the validator set on the
> blockchain changes. The core verification terminates successful
> crucially depends on the check [TP], that is, that the headers in
> *trustedStore* do not expire in the time needed to download more
> headers, which depends on the creation time of the headers in
> *trustedStore*. That is, termination of core verification is highly
> depending on the data stored in the blockchain.


> The current light client core verification protocol exploits that, in
> practice, changes in the validator set are rare. For instance,
> consider the following scenario.




#### No change in validator set

If on the blockchain the validator set of the block at height
*targetHeight* is equal to *startHeader.NextValidators*:
- there is one round trip in `IO` to download the header of height
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
