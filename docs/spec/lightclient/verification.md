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


#### **[TMBC-SOUND-DISTR-PossCommit]**:
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
called *Verifier*.  The *Verifier* communicates with a full node.  As
full nodes may be faulty, the light client has to check whether the
header it receives coincides with the one generated by Tendermint
consensus.

 To do verification (in particular the function
`VerifySingle`) checks based on these properties, the two 
 properties [[TMBC-VAL-CONTAINS-CORR]][TMBC-VAL-CONTAINS-CORR-link] and
[[TMBC-VAL-COMMIT]][TMBC-VAL-COMMIT-link]  formalize the checks done
 by this specification:
Given a trusted block *tb* and an untrusted block *ub* with a commit *cub*,
one has to check that *cub* is in *PossibleCommit(ub)*, and that *cub*
contains a correct node using *tb*.



# Part II - Sequential Definition of Fastsync Problem


## Informal Problem statement


Given a height *targetHeight* as an input, the *Verifier* stores a header of
height *targetHeight* locally.  This header is generated by the Tendermint
[blockchain][blockchain]. In particular, a header that was not
generated by the blockchain should never be stored.


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

The [fork detector][failuredetector] of the light client may help the correct full nodes to understand whether their header is a good one. 
Hence, in combination with the light client detector, the correct full nodes have the incentive to respond.
We can thus base liveness arguments on the assumption that correct full nodes reliably talk to the light client.



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
The light client has a local data structure called *trustedStore* that
contains headers.
**TODO:** make consistent with detector

#### **[LCV-DIST-PRIMARY]**:
The light client has a local variable *primary* that contains the Address (ID) of a full node.

#### **[LCV-DIST-INIT]**:
*trustedStore* is initialized with a header *trustedHeader* that was correctly 
generated by the Tendermint consensus.

### Temporal Properties

#### **[LCV-DIST-SAFE]**:
It is always the case that every header in *trustedStore* was generated by an instance of Tendermint consensus.

#### **[LCV-DIST-LIFE]**:
From time to time, a new instance of the verifier is called with a
height *targetHeight*. Each instance must eventually terminate. 
  - If
     - the full node (primary) with which the verifier communicates is
       correct, and 
     - *trustedStore* contains a header whose age is less than the
        trusting period, 

    then the instance adds a header *hd* with height
    *targetHeight* to *trustedStore* and it **terminates successfully**
	
  - Otherwise, it **terminates with failure**


> These definitions imply that if the primary is faulty, a header may or
> may not be added to *trustedStore*. In any case,
> [**[LCV-DIST-SAFE]**](#lcv-vc-inv) must hold.

> The invariant [**[LCV-DIST-SAFE]**](#lcv-vc-inv) and the liveness 
> requirement [**[LCV-DIST-LIFE]**](#lcv-vc-live)
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

```go
type SignedHeader struct {
    Header        Header
    Commit        Commit       
}
```

```
type LightBlock struct {
	signedHeader SignedHeader
    Validators  ValidatorSet
    NextValidators ValidatorSet
}
```	




### Inputs
- *trustedHeader*: the LightBlock verification starts from
- *primary*: peer address
- *targetHeight*: they height of the needed header

#### **[LCV-A-INIT]**:
- *trustedHeader.signedHeader* is from the blockchain
- *targetHeight > trustedHeader.height*

### Configuration Parameters

- *trustThreshold*: a float. Can be used if correctness should not be based on more voting power and 1/3.
- *trustingPeriod*: a time duration [**[TMBC-TIME_PARAMS]**][TMBC-TIME_PARAMS-link].
- *clockDrift*: a time duration. Correction parameter dealing with only approximately synchronized clocks.


### Variables

- *currentHeight*: initially *trustedHeader.Height*
- *nextHeight*: initially *targetHeight*
  > *nextHeight* should be thought of the "height of the next header we need
  > to download and verify"
- *trustedStore*: stores light blocks that have been downloaded and that
    passed verification. Initially it only contains *trustedHeader*.
  > Something like this was called *State* in ADR. Should be made
  >    consistent 
  >	with detector. We should decide what it contains: e.g., (i) set of headers,
  > (ii) set of TrustedState (iii) set of pairs: Trustedstate, address of
  > full node from which the lightlient downloaded the header
- *untrustedStore*: stores light blocks
   that have been downloaded and that failed
   verification, but may still be OK. Initially empty
- *Error*: error information. Initially nil.

- *headerToVerify*: a light block. Initially nil  
  **TODO:** decide whether passing of information should be done via
  shared variables  
  > used to hand the header to bisection. Might be a parameter
  > currently written by IO (or copied if a candidate is already 
  > in *untrustedStore*



### Auxiliary Functions

#### **[LCV-FUNC-REF]**:
- *refHeader*: is the header from *trustedStore* with the maximal
  height
  
  
#### **[LCV-FUNC-VERIF]**:
```go
func VerifySingle(untrustedVh LightBlock,
                  trustedVh LightBlock) (error)
```

- Expected precondition:
   - trustedVh.signedHeader.Commit is a commit is for the header 
     trustedVh.signedHeader.Header, i.e. it contains
     the correct hash of the header
   - the `Height` and `Time` of `trustedVh` are smaller than the Height and 
  `Time` of `untrustedVh`, respectively
   - the *untrustedVH.SignedHeader* is well-formed (passes the tests from
     [[block]]), and in particular
      - if the untrusted header `unstrustedVh` is the immediate 
	  successor  of  `trustedVh`, then it holds that
	      - *trustedVh.signedHeader.Header.NextValidators = 
		  untrustedVh.signedHeader.Header.Validators*, and
		  moreover, 
		  - *untrustedVh.signedHeader.Header.Commit* 
		     - contains signatures by more than two-thirds of the validators 
		     - contains no signature from nodes that are not in *trustedVh.signedHeader.Header.NextValidators*
- Expected postcondition: 
    - Returns `OK`:
        - if *untrustedVh* is the immediate successor of *trustedVh*,
          or otherwise,
        - if
		   -  signatures of a set of
             validators that have more than *max(1/3,trustThreshold)* of
             voting power in
             *trustedVh.signedHeader.Header.NextValidators*
			 is contained in *untrustedVh.SignedHeader.Commit*
           - (that is, header passes the tests [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link]  and [**[TMBC-VAL-COMMIT]**][TMBC-VAL-COMMIT-link])
	- Returns `CANNOT_VERIFY` 
        -  *untrustedVh* is *not* the immediate successor of
           *trustedVh*
		   and the  *max(1/3,trustThreshold)* threshold is not reached
        - (that is, if
	     [**[TMBC-VAL-CONTAINS-CORR]**][TMBC-VAL-CONTAINS-CORR-link] 
	     fails and header is does not violate the soundness
         checks [[block]]).
- Error condition: 
   - if precondition violated it returns a different error code

---
 
 ### Messages

**TODO:** 
 
### Remote Functions
  ```go
func Commit(addr Address, height int64) (SignedHeader, error)
```
- Implementation remark
   - RPC to peer with Address *addr*
   - Request message: **TODO**
   - Response message: **TODO**
- Expected precodnition
  - `height` is less than or equal to height of the peer
- Expected postcondition
  - if *addr* is correct: Returns the signed header of height `height`
  from the blockchain
  - if *addr* is faulty: Returns a signed header with arbitrary content
- Error condition
   * if *n* is correct: precondition violated **TODO:** mention message
   * if *n* is faulty: arbitrary error

---


 ```go
func Validators(addr Address, height int64) (ValidatorSet, error)
```
- Implementation remark
   - RPC to peer with Address *addr*
   - Request message: **TODO**
   - Response message: **TODO**
- Expected precodnition
  - `height` is less than or equal to height of the peer
- Expected postcondition
  - if *addr* is correct: Returns the validator set of height `height`
  from the blockchain
  - if *addr* is faulty: Returns a validator set with arbitrary content
- Error condition
   * if *n* is correct: precondition violated **TODO:** mention message
   * if *n* is faulty: arbitrary error

---


## Core Verification

### Outline

The protocols is described in terms of the following functions.

- `getHeaderData` is called to do IO, that is, 
  download the next header, and validator sets
- `VerifyBisection` calls `verifySingle` to see whether a new header can
  be trusted. If not, it does bisection by updating the nextheight
  variable
  

In the current Rust architecture we consider a sequential flow, that
is, execution of a loop where first `getHeaderData(primary)` and then
`VerifyBisection` are invoked (`VerifyBisection` calls `verifySingle`).
 `getHeaderData` only needs to be called if *untrustedStore* does
 not contain a header of height *nextHeight*, otherwise the header is
 just moved from *untrustedStore* to *headerToVerify*.

### Termination Conditions

#### **[LCV-TERM-EXPIRED]**
If *refHeader.signedHeader.header.time < now - trustingPeriod* 
set *Error = EXPIRED* then **terminate with failure**.

#### **[LCV-TERM-SUCCESS]**
If *currentHeight = targetHeight* then **terminate successfully**


### Details



```go
func getHeaderData(addr Address)
```
-  Implementation remark
   - Used to communicate with a full node *n* at address *addr* via 
     RPCs `Commit` and `Validators` 
   - The only function that makes external calls!
   - This function make externals RPC calls to the full node; 
      - `Validators(primary,nextHeight)`
	  - `Validators(primary,nextHeight+1)` 
	  - `Commit (primary,nextHeight)`
   
- Expected precondition
  - *nextheight > refHeader.signedHeader.Header.Height*
- Expected postcondition: 
  - If *n* is correct, it returns the following data:
    - *headerToVerify.signedHeader* is a signed header consistent with
      the blockchain
    - *headerToVerify.Validators* is the validator set of the
      blockchain at height *nextheight*
    - *headerToVerify.NextValidators* is the validator set of the
      blockchain at height *nextheight + 1*
    - *headerToVerify.signedHeader.Header.Time < now + clockDrift*
  - If *n* is faulty, returns arbitrary 
  - [LCV-INV-VAL] (if *n* is faulty or not)
- Error conditions
  - precondition violated
  - If the field time of the returned signed header is greater than or
    equal to `now + clockDrift`, then set *error = InvalidHeaderTime*
    and **terminate with failure**.
  - If *n* is faulty

---


```go
func VerifyBisection {
    err := VerifySingle(trustedHeader, headerToVerify)
    if err == OK { 
	  trustedStore.add(headerToVerify)
	  // **TODO:** reset headerToVerify to nil?
      currentHeight = nextHeight
	  nextHeight = targetHeight
    } else if err = CANNOT_VERIFY{
	  untrustedStore.add(headerToVerify)
      compute pivot // (currentHeight + nextHeight) / 2
      nextHeight = pivot
    }
  }
}
```
- Expected precondition
  - height of *headerToVerify* is *nextHeight*
  - height of *trustedHeader* is *currentHeight*
- Expected postcondition
  - *nextHeight <= targetHeight*
  - *nextHeight > currentHeight*
  - height of *trustedHeader* is *currentHeight*
- Error conditions 
  - IF `VerifySingle` returns a error code *code* different from OK or
  CANNOT_VERIFY then set *error = code*
    and **terminate with failure**.
	
> **TODO:** I encoded a bisection which is the most simple to
> describe. With the current variables, we could also try to verify all
> headers in *untrustedStore* which uses more of the already
> downloaded headers.	
	
---










### Algorithm Invariants

### **[LCV-INV-VAL]**
At all times
- *headerToVerify.Validators = hash(headerToVerify.signedHeader.Header.Validators)*
- *headerToVerify.NextValidators = hash(headerToVerify.signedHeader.Header.NextValidators)*





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
[TMBC-SOUND-DISTR-PossCommit-link]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#tmbc-sound-distr-posscommit


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
