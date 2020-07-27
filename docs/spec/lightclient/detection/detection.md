***This an unfinished draft. Comments are welcome!***

This document contains:

- the outcome of recent discussion
- a sketch of the light client supervisor to provide the context in
  which fork detection happens
- a discussion about lightstore semantics
- a draft of the light node fork detection including "proof of fork"
  definition, that is, the data structure to submit evidence to full
  nodes.
  

## Results of Discussions and Decisions 

- Generating a minimal proof of fork is too costly at the light client
    - we do not know all lightblocks from the primary
	- therefore there are many scenarios. we might even need to ask
      the primary again for additional lightblocks to isolate the
      branch. 
> For instance, the light node starts with block at height 1 and the
> primary provides a block of height 10 that the light node can
> verify immediately. In cross-checking, a secondary now provides a
> conflicting header b10 of height 10 that needs another header b5
> of height 5 to
> verify. Now, in order for the light node to convince the primary:
>   - The light node cannot just sent b5, as it is not clear whether
>     the fork happened before or after 5
>   - The light node cannot just send b10, as the primary would also 
>     need  b5 for verification
>   - In order to minimize the evidence, the light node may try to
>     figure out where the branch happens, e.g., by asking the primary
>     for height 5 (it might be that more queries are required, also
>     to the secondary. However, assuming that in this scenario the
>     primary is faulty it may not respond.
	  
	  As the main goal is to catch misbehavior of the primary,
      evidence generation and punishment must not depend on their
      cooperation. So the moment we have proof of fork (even if it
      contains several light blocks) we should submit right away.


- decision: "full" proof of fork consists of two traces that origin in the
  same lightblock and lead to conflicting headers of the same height.
  
- For submission of proof of fork, we may do some optimizations, for
  instance, we might just submit  a trace of lightblocks that verifies a block
  different from the one the full node knows (we do not send the trace
  the primary gave us back to the primary)

- The light client attack is via the primary. Thus we try to
  catch if the primary installs a bad light block
    - We do not check secondary against secondary
    - For each secondary, we check the primary against one secondary


# Light Client Sequential Supervisor

**TODO:** decide where (into which specification) to put the
following:


We describe the context on which the fork detector is called by giving
a sequential version of the supervisor function.
Roughly, it alternates two phases namely:
   - Light Client Verification. As a result, a header of the required
     height has been downloaded from and verified with the primary
   - Light Client Fork Detections. As a result the header has been
     cross-checked with the secondaries. In case the is a fork we
     submit "proof of fork" and exit.
   
	 




#### **[LC-FUNC-SUPERVISOR.1]:**

```go
func Sequential-Supervisor () (Error) {
    loop {
	    // get the next height
        nextheight := input();
		
		// Verify
        result := NoResult;
        while result != ResultSuccess {
            lightStore,result := VerifyToTarget(primary, lightStore, nextheight);
            if result == ResultFailure {				
				// pick new primary (promote a secondary to primary)
				/// and delete all lightblocks above
	            // LastTrusted (they have not been cross-checked)
	            Replace_Primary();
			}
        }
		
		// Cross-check
        PoFs := Forkdetector(lightStore, PoFs);
        if PoFs.Empty {
		    // no fork detected with secondaries, we trust the new
			// lightblock
            LightStore.Update(testedLB, StateTrusted);
        } 
        else {
		    // there is a fork, we submit the proofs and exit
            for i, p range PoFs {
                SubmitProofOfFork(p);
            } 
            return(ErrorFork);
        }
    }
}
```
**TODO:** finish conditions
- Implementation remark
- Expected precondition
    - *lightStore* initialized with trusted header
	- *PoFs* empty
- Expected postcondition
    - runs forever, or
	- is terminated by user and satisfies LightStore invariant, or **TODO**
	- has submitted proof of fork upon detecting a fork
- Error condition
    - none
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



> This is where the actual specification is going to start

# Fork detector

A detector (or detector for short) is a mechanism that expects as
input a header with some height *h*, connects to different Tendermint
full nodes, requests the header of height *h* from them, and then
cross-checks the headers and the input header.

There are two foreseeable use cases:

1) strengthen the light client: If a light client accepts a header
*hd* (after performing skipping or sequential verification), it can
use the  detector to probe the system for conflicting headers and
increase the trust in *hd*. Instead of communicating with a single
full node, communicating with several full nodes shall increase the
likelihood to be aware of a fork (see [[accountability]] for
discussion about forks) in case there is one.

2) to support fork accountability: In the case when more than 1/3 of
the voting power is held by faulty validators, faulty nodes may
generate two conflicting headers for the same height. The goal of the
detector is to learn about the conflicting headers by probing
different full nodes. Once a detector has two conflicting headers,
these headers are evidence of misbehavior. A natural extension is to
use the detector within a monitor process (on a full node) that calls
the detector on a sample (or all) headers (in parallel). (If the
sample is chosen at random, this adds a level of probabilistic
reasoning.) If conflicting headers are found, they are evidence that
can be used for punishing processes.

In this document we will focus onn strengthening the light client, and
leave other uses of the detection mechanism (e.g., when run on a full
node) to the future.


## Context of this document

The light client verification specification [[verification]] is
designed for the Tendermint failure model (1/3 assumption)
[TMBC-FM-2THIRDS]. It is safe under this assumption, and live
if it can reliably (that is, no message loss, no duplication, and
eventually delivered) and timely communicate with a correct full node. If
this assumption is violated, the light client can be fooled to trust a
header that was not generated by Tendermint consensus.

This specification, the fork detector, is a "second line of defense",
in case the 1/3 assumption is violated. Its goal is to detect fork (conflicting headers) and collect
evidence. However, it is impractical to probe all full nodes. At this
time we consider a simple scheme of maintaining an address book of
known full nodes from which a small subset (e.g., 4) are chosen
initially to communicate with. More involved book keeping with
probabilistic guarantees can be considered at later stages of the
project.

The light client maintains a simple address book containing addresses
of full nodes that it can pick as primary and secondaries.  To obtain
a new header, the light client first does [verification](verification)
with the primary, and then cross-checks the header with the
secondaries using this specification.


### Tendermint Consensus and Forks

#### **[TMBC-GENESIS.1]**
Let *Genesis* be the agreed-upon initial block (file).

#### **[TMBC-FUNC.1]**
> **TODO** be more precise. +2/3 of b.NextV = c.Val signed c. For now
> the following will do:

Let b and c be two light blocks
with *b.Header.Height + 1 = c.Header.Height*. We define **signs(b,c)**
    iff `ValidAndVerified(b, c)` 

> **TODO** be more precise. +1/3 of b.NextV signed c. For now
> the following will do:

Let *b* and *c* be two light blocks. We define **supports(b,c,t)** 
    iff `ValidAndVerified(b, c)` at time *t*


> The following formalizes that *b* was properly generated by
> Tendermint; *b* can be traced back to genesis

#### **[TMBC-SEQ-ROOTED.1]**
Let *b* be a light block. 
We define *sequ-rooted(b)* iff for all i, 1 <= i < h = b.Header.Height,
there exist light blocks a(i) s.t.
   - *a(1) = Genesis* and
   - *a(h) = b* and
   - *signs( a(i) , a(i+1) )*.

> The following formalizes that *c* is trusted based on *b* in
> skipping verification. Observe that we do not require here (yet)
> that *b* was properly generated.

#### **[TMBC-SKIP-ROOT.1]**
Let *b* and *c* be light blocks. We define *skip-root(b,c,t)* if at
time t there exists an *h* and a sequence *a(1)*, ... *a(h)* s.t.
   - *a(1) = b* and
   - *a(h) = c* and
   - *supports( a(i), a(i+1), t)*, for all i, *1 <= i < h*.

> **TODO** In the above we might use a sequence of times t(i). Not sure.

> **TODO:** I believe the following definition
> corresponds to **Slashable fork** in
> [forks][tendermintfork]. Please confirm!  
> Observe that sign-skip-match  is even defined if there is a fork on
> the chain.

#### **[TMBC-SIGN-SKIP-MATCH.1]**
Let *a*, *b*, *c*, be light blocks and *t* a time, we define 
*sign-skip-match(a,b,c,t) = true* iff
   - *sequ-rooted(a)* and
   <!-- - *sequ-rooted(b)* and -->
   - *b.Header.Height = c.Header.Height* and
   - *skip-root(a,b,t)*
   - *skip-root(a,c,t)*

implies *b = c*.

----

#### **[TMBC-SIGN-FORK.1]**

If there exists three light blocks a, b, and c, with 
*sign-skip-match(a,b,c,t) =
false* then we have a *slashable fork*.

We call *a* the bifurcation block of the fork.
----


> **TODO:** I think the following definition is
> the intuition behind **main chain forks**
> in the document on [forks][tendermintfork]. However, main chain
> forks were defined more operational "forks that are observed by
> full nodes as part of normal Tendermint consensus protocol". Please
> confirm! 
  
#### **[TMBC-SIGN-UNIQUE.1]**
Let *b* and *c* be  light blocks, we define *sign-unique(b,c) =
true* iff
   - *b.Header.Height =  c.Header.Height* and
   - *sequ-rooted(b)* and
   - *sequ-rooted(c)*
   
implies *b = c*.

If there exists two light blocks b and c, with *sign-unique(b,c) =
false* then we have a *fork on the chain*.

> The following captures what I believe is called a light client fork
> in our discussions. There is no fork on the chain up to the height 
> of block b. However, c is of that height, is different, and passes skipping
> verification  
> Observe that a light client fork is a special case of a slashable
> fork. 


#### **[TMBC-LC-FORK.1]**
Let *a*, *b*, *c*, be light blocks and *t* a time. We define
*light-client-fork(a,b,c,t)* iff
   - *sign-skip-match(a,b,c,t) = false* and
   - *sequ-rooted(b)* and
   - *b* is "unique", that is, for all *d*,  *sequ-rooted(d)* and
     *d.Header.Height=b.Header.Height* implies *d = b*


> Finally, let's also define bogus blocks that have no support.
> Observe that bogus is even defined if there is a fork on the chain.
> Also, for the definition it would be sufficient to restrict *a* to 
> *a.height < b.height* (which is implied by the definitions which
> unfold until *supports()*.

#### **[TMBC-BOGUS.1]**
Let *b* be a light block and *t* a time. We define *bogus(b,t)* iff
  - *sequ-rooted(b) = false* and
  - for all *a*, *sequ-rooted(a)* implies *skip-root(a,b,t) = false*
  
  
  

> Relation to [fork accountability][accountability]: F1, F2, and F3
> (equivocation, amnesia, back to the past) can lead to a fork on the
> chain and to a light client fork.  
> F4 and F5 (phantom validators, lunatic) cannot lead to a fork on the
> chain but to a light client
> fork if *t+1 < f < 2t+1*.  
> F4 and F5 can also lead to bogus blocks



### Informal Problem statement

> We put tags to informal problem statements as there is no sequential
> specification.

The following requirements are operational in that they describe how
things should be done, rather than what should be done. However, they
do not constitute temporal logic verification conditions. For those,
see [LCD-DIST-*] below.


#### **[LCD-IP-STATE.1]**

The detector works on a LightStore that contains LightBlocks in one of 
the state `StateUnverified`, ` StateVerified`, `StateFailed`, or
`StateTrusted`.


#### **[LCD-IP-Q.1]**

Whenever the light client verifier performs `VerifyToTarget` with the
primary and returns with
`(lightStore, ResultSuccess)`, the
 detector should query the secondaries by calling `FetchLightBlock` for height
 *LightStore.LatestVerified().Height* remotely.  
Then, 
the detector returns the set of all headers *h'* downloaded from
secondaries that satisfy
 - *h'* is different from *LightStore.LatestVerified()*
 - *h'* is a (light) fork.


#### **[LCD-IP-PEERSET.1]**

Whenever the detector observes *detectable misbehavior* of a full node
from the set of Secondaries it should be replaced by a fresh full
node.  (A full node that has not been primary or secondary
before). Detectable misbehavior can be
- a timeout
- The case where *h'* is different
from *LightStore.LatestVerified()* but *h'* is not a fork, that is, if
*h'* is bogus. In other words, the
secondary cannot provide a sequence of light blocks that constitutes
proof of *h'*.





## Assumptions/Incentives/Environment

It is not in the interest of faulty full nodes to talk to the 
detector as long as the  detector is connected to at least one
correct full node. This would only increase the likelihood of
misbehavior being detected. Also we cannot punish them easily
(cheaply). The absence of a response need not be the fault of the full
node. 

Correct full nodes have the incentive to respond, because the 
detector may help them to understand whether their header is a good
one. We can thus base liveness arguments of the  detector on
the assumptions that correct full nodes reliably talk to the 
detector.


### Assumptions

#### **[LCD-A-CorrFull.1]**

At all times there is at least one correct full
node among the primary and the secondaries.

**Remark:** Check whether [LCD-A-CorrFull.1] is not needed in the end because
the verification conditions [LCD-DIST-*] have preconditions on specific
cases where primary and/or secondaries are faulty.

#### **[LCD-A-RelComm.1]**

Communication between the  detector and a correct full node is 
reliable and bounded in time. Reliable communication means that
messages are not lost, not duplicated, and eventually delivered. There
is a (known) end-to-end delay *Delta*, such that if a message is sent
at time *t* then it is received and processed by time *t + Delta*.
This implies that we need a timeout of at least *2 Delta* for remote
procedure calls to ensure that the response of a correct peer arrives
before the timeout expires.




## (Distributed) Problem statement

> As the fork detector from the beginning is there to reduce the
> impact of faulty nodes, and faulty nodes imply that there is a
> distributed system, there is no sequential specification.

The  detector gets as input a lightstore *lightStore*.
Let *h-verified = lightStore.LatestVerified().Height* and
     *h-trust=lightStore.LatestTrusted().Height* (see
     [LCV-DATA-LIGHTSTORE]).
It queries the secondaries for  headers at height *h-verified*.
The  detector returns a set *Forks*, and should satisfy the following
     temporal formulas:


#### **[LCD-DIST-INV.1]**

If there is no fork at height *h-verified*, then the detector should
return the empty set.


**TODO:** be precise about what a fork is. 

**TODO:** add requirements about stateTrusted

#### **[LCD-DIST-LIVE-FORK.1]**

If there is a fork at height *h*, with *h-trust < h <= h-verified*, and
there are two correct full nodes *i* and *j* that are
  - on different branches, and
  - primary or secondary,

then the  detector eventually outputs the fork.

**TODO:** We can weaken the above to "the (not-necessarily correct)
primary provided branch A, and a correct secondary is on branch B". I
prefer the above as it is slightly less operational.


> #### **[LCD-REQ-REP.1]**
> If the  detector observes two conflicting headers for height *h*, 
> it should try to verify both. If both are verified it should report evidence.
> If the primary reports header *h* and a secondary reports header *h'*,
>     and if *h'* can be verified based on common root of trust, then
>     evidence should be generated; 
> By verifying we mean calling `VerifyToTarget` from the
> [[verification]] specification.

## Definitions

- A fixed set of full nodes is provided in the configuration upon
     initialization. Initially this set is partitioned into
    -  one full node that is the *primary* (singleton set),
	-  a set *Secondaries* (of fixed size, e.g., 3),
	-  a set *FullNodes*.
- A set *FaultyNodes* of nodes that the light client suspects of being faulty; it is initially empty

- *Lightstore* as defined in the [verification specification][verification].




#### **[LCD-INV-NODES.1]:**
The detector shall maintain the following invariants:
   - *FullNodes \intersect Secondaries = {}*
   - *FullNodes \intersect FaultyNodes = {}*
   - *Secondaries \intersect FaultyNodes = {}*
   
and the following transition invariant
   - *FullNodes' \union Secondaries' \union FaultyNodes' = FullNodes \union Secondaries \union FaultyNodes*


#### **[LCD-INV-TRUSTED-AGREED.1]:**

**TODO** The primary and the secondary agree on LatestTrusted.

## Solution

### Data Structures

Lightblocks and LightStores are
defined at [LCV-DATA-LIGHTBLOCK.1] and [LCV-DATA-LIGHTSTORE.1]. See the [verification specification][verification] for details.

The following data structure defines a **proof of fork**. Following
[TMBC-SIGN-FORK.1], we require two blocks *b* and *c* for the same
height that can both be verified from a common root block *a* (using
the skipping or the sequential method).

**TODO:** move this discussion up to beginning of spec!

> Observe that just two blocks for the same height are not
> sufficient. One of the blocks may be bogus [TMBC-BOGUS.1] which does
> not constitute slashable behavior. 

> Which leads to the question whether the light node should try to do
fork detection on its initial block (from subjective
initialization). This could be done by doing backwards verification
(with the hashes) until a bifurcation block is found. 
While there are scenarios where a
fork could be found, there is also the scenario where a faulty full
node feeds the light node with bogus light blocks and forces the light
node to check hashes until a bogus chain is out of the trusting period.
As a result, the light client
should not try to establish a fork for its initial header.

#### **[LCV-DATA-POF.1]**:
```go
type LightNodeProofOfFork struct {
    TrustedBlock      LightBlock
    PrimaryTrace      []LightBlock
    SecondaryTrace    []LightBlock
}
```

> [LCV-DATA-POF.1] mirrors the definition [TMBC-SIGN-FORK.1]:
> *TrustedBlock* corresponds to *a*, and *PrimaryTrace* and *SecondaryTrace*
> are traces to two blocks *b* and *c*. The traces establish that both
> *skip-root(a,b,t)* and *skip-root(a,c,t)* are satisfied.





<!-- ```go -->
<!-- // info about the LC's last trusted block -->
<!-- type TrustedBlockInfo struct { -->
<!--   Height              int -->
<!--   BlockID             BlockID -->
<!-- }  -->
<!-- ``` -->

#### **[LCV-DATA-POFSTORE.1]**:

Proofs of Forks are stored in a structure which stores all  proofs
generated during detection.

```go
type PoFStore struct {
	...
}
```


In additions to the functions defined in 
the [verification specification][verification], the 
LightStore exposes the following function

#### **[LCD-FUNC-SUBTRACE.1]:**
```go
func (ls LightStore) Subtrace(from int, to int) LightStore
```
- Expected postcondition
   - returns a lightstore that contains all lightblocks *b* from *ls*
     that satisfy: *from < b.Header.Height <= to*
----




### Inter Process Communication


```go
func FetchLightBlock(peer PeerID, height Height) LightBlock
```
See the [verification specification][verification] for details.


#### **[LCD-FUNC-SUBMIT.1]:**
```go
func SubmitProofOfFork(pof LightNodeProofOfFork) Result
```
- Implementation remark
- Expected precondition
    - none
- Expected postcondition
    - submit evidence to primary and the secondary in *pof*, that is,
      to
	     - `pof.PrimaryTrace[1].Provider`
	     - `pof.SecondaryTrace[1].Provider`
    - **QUESTION** minimize data? We could submit to the primary only
      the trace of the secondary, and vice versa. Do we need to spell
      that out here? (Also, by [LCD-INV-TRUSTED-AGREED.1], we do not
      need to send `pof.TrustedBlock`)
	- **FUTURE WORK:** we might send *pof* to primary or all
      secondaries or broadcast to all full nodes. However, in evidence
      detection this might need that a full node has to check a *pof*
      where both traces are not theirs. This leads to more complicated
      logic at the full node, which we do not need right now.

- Error condition
    - none

### Auxiliary Functions (Local)

#### **[LCD-FUNC-CROSS-CHECK.1]:**

```go
func CrossCheck(peer PeerID, testedLB LightBlock) (result) {
	sh := FetchLightBlock(peer, testedLB.Height);
		// as the check below only needs the header, it is sufficient
		// to download the header rather than the LighBlock
    if testedLB.Header == sh.Header {
	    return OK
	}
	else {
	    return DoesNotMatch
	}
}
```
- Implementation remark
- Expected precondition
- Expected postcondition
- Error condition


#### **[LCD-FUNC-REPLACE-PRIMARY.1]:**
```go
Replace_Primary()
```
- Implementation remark
    - the primary is replaced by a secondary, and lightblocks above
      trusted blocks are removed
	- to maintain a constant size of secondaries, at this point we
      might need to 
	     - pick a new secondary nsec
		 - maintain [LCD-INV-TRUSTED-AGREED.1], that is,
		    - call `CrossCheck(nsec,lightStore.LatestTrusted()`.
              If it matches we are OK, otherwise
			     - we might just repeat with another full node as new secondary
				 - try to do fork detection from some possibly old
                   lightblock in store. (Might be the approach for the
                   light node that assumes to be connected to correct
                   full nodes only from time to time)
	  
- Expected precondition
    - *FullNodes* is nonempty
	
- Expected postcondition
    - *primary* is moved to *FaultyNodes*
    - all lightblocks with height greater than
      lightStore.LatestTrusted().Height are removed from *lightStore*.
    - a secondary *s* is moved from *Secondaries* to primary
> this ensures that *s* agrees on the Last Trusted State

- Error condition
    - if precondition is violated

**TODO:** pass in the lightstore to the replace functions

#### **[LCD-FUNC-REPLACE-SECONDARY.1]:**
```go
Replace_Secondary(addr Address)
```
- Implementation remark
     - maintain [LCD-INV-TRUSTED-AGREED.1], that is,
		 - call `CrossCheck(nsec,lightStore.LatestTrusted()`.
           If it matches we are OK, otherwise
			   - we might just repeat with another full node as new secondary
			   - try to do fork detection from some possibly old
                   lightblock in store. (Might be the approach for the
                   light node that assumes to be connected to correct
                   full nodes only from time to time)

- Expected precondition
    - *FullNodes* is nonempty
- Expected postcondition
    - addr is moved from *Secondaries* to *FaultyNodes*
    - an address *a* is moved from *FullNodes* to *Secondaries*
- Error condition
    - if precondition is violated



### From the verifier
```go
func VerifyToTarget(primary PeerID, lightStore LightStore,
                    targetHeight Height) (LightStore, Result)
```
See the [verification specification][verification] for details.



## Solution

### Shared data of the light client
- a pool of full nodes *FullNodes* that have not been contacted before
- peer set called *Secondaries*
- primary
- lightStore


### Outline

The problem laid out is solved by calling  the function `ForkDetector`
     with a lightstore that contains a light block that has just been
     verified by the verifier. 
	 




- **TODO:** We should clarify what is the expectation of VerifyToTarget so if it
  returns TimeoutError it can be assumed faulty. I guess that
  VerifyToTarget with correct full node should never terminate with
  TimeoutError.

- **TODO:** clarify EXPIRED case. Can we always punish? Can we give sufficient
  conditions. 
  
	  


### Fork Detector


#### **[LCD-FUNC-DETECTOR.1]:**
```go
func ForkDetector(ls LightStore, PoFs PoFStore) 
{
	testedLB := LightStore.LatestVerified()
	for i, secondary range Secondaries {
	    if OK = CrossCheck(secondary, testedLB) {
			// header matches. we do nothing.
		} 
		else {
			// [LCD-REQ-REP]
			// header does not match. there is a situation.
			// we try to verify sh by querying s
			// we set up an auxiliary lightstore with the highest
			// trusted lightblock and the lightblock we want to verify
			auxLS.Init
			auxLS.Update(LightStore.LatestTrusted(), StateVerified);
			auxLS.Update(sh,StateUnverified);
			LS,result := VerifyToTarget(secondary, auxLS, sh.Header.Height)
			if (result = ResultSuccess || result = EXPIRED) {
				// we verified header sh which is conflicting to hd
				// there is a fork on the main blockchain.
				// If return code was EXPIRED it might be too late
				// to punish, we still report it.
				pof = new LightNodeProofOfFork;
				pof.TrustedBlock := LightStore.LatestTrusted()	
				pof.PrimaryTrace := 
				    LightStore.Subtrace(LightStore.LatestTrusted().Height, 
					                    testedLB.Height);
				pof.SecondaryTrace := 
				    auxLS.Subtrace(LightStore.LatestTrusted().Height, 
					               testedLB.Height);
				PoFs.Add(pof);
			} 
			else {
				// secondary might be faulty or unreachable
				// it might fail to provide a trace that supports sh
				// or time out
				Replace_Secondary(secondary)
			}
		}
	}
	return PoFs
}
```
- Expected precondition
	- Secondaries initialized and non-empty
	- `PoFs` initialized and empty
- Expected postcondition
    - satisfies [LCD-DIST-INV.1], [LCD-DIST-LIFE-FORK.1]
	- removes faulty secondary if it reports wrong header
	- **TODO** proof of fork
- Error condition
    - fails if precondition is violated
	- fails if [LCV-INV-TP] is violated (no trusted header within
      trusting period
----


## Correctness arguments



#### Argument for [LCD-DIST-INV]

**TODO**


#### Argument for [LCD-DIST-LIFE-FORK]
**TODO**





# References

> links to other specifications/ADRs this document refers to


[[verification]] The specification of the light client verification.

[[tendermintfork]] Tendermint fork detection and accountability

[[accountability]] Fork accountability

[TMBC-FM-2THIRDS-linkVDD]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md#**[TMBC-FM-2THIRDS-link]**:

[TMBC-FM-2THIRDS-link]: https://github.com/tendermint/spec/blob/master/spec/consensus/light-client/verification.md


[block]: https://github.com/tendermint/spec/blob/master/spec/blockchain/blockchain.md

[blockchain]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md

[lightclient]: https://github.com/interchainio/tendermint-rs/blob/e2cb9aca0b95430fca2eac154edddc9588038982/docs/architecture/adr-002-lite-client.md

[verificationVDD]: https://github.com/informalsystems/VDD/blob/master/lightclient/failuredetector.md

[verification]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md

[accountability]: https://github.com/tendermint/spec/blob/master/spec/consensus/light-client/accountability.md

[tendermintfork]: https://docs.google.com/document/d/1xjyp5JOPt7QfHem1AFEaowBH2Plk0IHACWtYFXFvO7E/edit#heading=h.th2369ptc2ve
