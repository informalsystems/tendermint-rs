
# Proof of Fork handling

Light client executes verification and detection procedures with a primary and a set of witnesses. 
After light block lb is verified at some height h with a primary it is then checked with each
witness. If a witness returns a different light block for the height h than lb we run bisection
with the witness starting from the latest trusted header. Note that we are primarily interested in the
case where a primary is faulty and the witness is a correct node. By a correct full node we assume
a full node that is on the correct (main) chain and which is responsive (replies to the requests
timely and with a correct response). Executing bisection with a witness assumes executing verification
logic starting from the latest trusted header until the point of bifurkation. If the bisection succesfully 
terminates, we have proved that there is a fork and the trace of light blocks from the primary (starting
with the latest trusted block and ending with the point of bifurkation) is submitted to a witness. We also 
submit the corresponding trace obtained from the witness to a primary. Note that in case a witness is faulty
he might not be cooperative and the bisection will time out or fail to terminate bisection successfully. 

## Facts

1. Starting from a trusted header (point of trust) there are two traces of light blocks that ends in a different
header at bifurkation point. 
2. As we assume that a witness is a correct full node, that implies that starting from a trusted header a 
misbehavior has taken place (Tendermint security model is broken), i.e., there are at least +1/3 of faulty 
validators that misbehaved and created a fork. 

Example 1 

1. point of trust is header at height 1 with valset p1, p2, p3 and p4
2. at primary there is a header at height 4 with valset p1, p2, p5 and p6. We assume that
p1 and p2 misbehaved and signed different header for height 4 at two branches. On the witness 
there is a header at height 4 with a different header. It could have completely different 
valset p7, p8, p9 and p10. 

Main branch:

at h1 we have (p1, p2, p3, p4) -> at h2 we have (p7, p8, p9, p10) -> at h3 we have (p7, p8, p9, p10) -> at h4 we have (p7, p8, p9, p10) 
 
Fork branch:

at h1 we have (p1, p2, p3, p4) -> at h4 we have (p1, p2, p5, p6)

This is an example of the lunatic attack by p1 and p2. In this case we will create 
lunatic evidence for p1 and p2 as they are part of the trusted val set, but not for p5 and p6 as 
we are not certain if they are bonded. 

Example 2

Main branch:

at h1 we have (p1, p2, p3, p4) -> at h2 we have (p7, p8, p9, p10) -> at h3 we have (p7, p8, p9, p10) -> at h4 we have (p7, p8, p9, p10) 
 
Fork branch:

at h1 we have (p1, p2, p3, p4) -> at h2 we have (p7, p8, p9, p10) -> at h3 we have (p11, p12, p13, p14) -> at h4 we have (p11, p12, p13, p14)

p7 and p8 attacked the system at h2 that has led to a different valset being selected for the height 3. In the worst
case none of these validators was bonded. At heights h3 and h4 no misbehaviour has been made and it is also not slashable.
Therefore, we need to create evidence of misbehavior at the biforkation point. 


 

### Data Structures

Lightblocks and LightStores are
defined at [LCV-DATA-LIGHTBLOCK.1] and [LCV-DATA-LIGHTSTORE.1]. See the [verification specification][verification] for details.

> The following data structure [LCV-DATA-POF.1]
> defines a **proof of fork**. Following
> [TMBC-SIGN-FORK.1], we require two blocks *b* and *c* for the same
> height that can both be verified from a common root block *a* (using
> the skipping or the sequential method).

> [LCV-DATA-POF.1] mirrors the definition [TMBC-SIGN-FORK.1]:
> *TrustedBlock* corresponds to *a*, and *PrimaryTrace* and *SecondaryTrace*
> are traces to two blocks *b* and *c*. The traces establish that both
> *skip-root(a,b,t)* and *skip-root(a,c,t)* are satisfied.


#### **[LCV-DATA-POF.1]**:
```go
type LightNodeProofOfFork struct {
    TrustedBlock      LightBlock
    PrimaryTrace      []LightBlock
    SecondaryTrace    []LightBlock
}
```

The trace will have the following data structure: 

```go
type LightNodeProofOfFork struct {
  Trace []LightBlock
}
```

```go
type LightBlock struct {
		Header          Header
		Commit          Commit
		Validators      ValidatorSet
		NextValidators  ValidatorSet
		Provider        PeerID
}
```
### Validating Proof of Fork

Once a correct full node receives a `LightNodeProofOfFork` it first validates it:

```go
func IsValid(pof LightNodeProofOfFork, bc Blockchain) boolean {

    trusted = pof.Trace[0].Header
    pointOfBifurkation = pof.Trace[len(pof.Trace)-1].Header 
    
    // In valid proof of fork, trace starts with the header equal to the corresponding 
    // header on chain and ends with the different header.
    if trusted != bc[trusted.Height] or 
       poinOfBifurkation == bc[poinOfBifurkation.Height].Header {
        return false    
    }
    
    for i in 1..len(pof.Trace)-1 {
        untrusted = pof.Trace[i].Header
        // Note that trustingPeriod in ValidAndVerified is set to UNBONDING_PERIOD
        verdict = ValidAndVerified(trusted, untrusted) 
        if verdict == OK {
            trusted = untrusted
        }
        else return false   
    }
    return true
}
```

### Evidence creation

```go
type LunaticValidatorEvidence struct {
	Header             types.Header
	Vote               types.Vote
}
```

```go
type DuplicateVoteEvidence struct {
	VoteA             types.Vote
	VoteB             types.Vote	
}
```

```go
type PotentialAmnesiaEvidence struct {
	ConflictingBlock   LightBlock
    ValidatorAddress   Address
}
```

```go
func getMinBifurcationPoint(pof LightNodeProofOfFork, bc Blockchain) (int, LightBlock) {
    trusted = pof.Trace[i].Header
    conflicting = nil
    for i in 1..len(pof.Trace)-1 {
        header = pof.Trace[i].Header
        if header != bc[header.Height].Header {
            conflicting = header 
            height = trusted.Height
            return (height, pof.Trace[i])
        }
        else {
            trusted = header
        }
    }
    panic
}    
```

```go
type Commit struct {
	Height     int64
	Round      int
	BlockID    BlockID
	Signatures []CommitSig
}
```

```go
type BlockIDFlag byte

const (
	// BlockIDFlagAbsent - no vote was received from a validator.
	BlockIDFlagAbsent BlockIDFlag = 0x01
	// BlockIDFlagCommit - voted for the Commit.BlockID.
	BlockIDFlagCommit = 0x02
	// BlockIDFlagNil - voted for nil.
	BlockIDFlagNil = 0x03
)

type CommitSig struct {
	BlockIDFlag      BlockIDFlag
	ValidatorAddress Address
	Timestamp        time.Time
	Signature        []byte
}
```

```go
type Header struct {
	// basic block info
	Version  Version
	ChainID  string
	Height   int64
	Time     Time

	// prev block info
	LastBlockID BlockID

	// hashes of block data
	LastCommitHash []byte // commit from validators from the last block
	DataHash       []byte // MerkleRoot of transaction hashes

	// hashes from the app output from the prev block
	ValidatorsHash     []byte // validators for the current block
	NextValidatorsHash []byte // validators for the next block
	ConsensusHash      []byte // consensus params for current block
	AppHash            []byte // state after txs from the previous block
	LastResultsHash    []byte // root hash of BeginBlock events, root hash of all results from the txs from the previous block, and EndBlock events

	// consensus info
	EvidenceHash    []byte // evidence included in the block
	ProposerAddress []byte // original proposer of the block
```

```go
func createLunaticEvidences(height int, conflictingBlock LightBlock, bc Blockchain) []LunaticValidatorEvidence {
   evidences = []LunaticValidatorEvidence
   trusted = bc[conflicting.Header.Height].Header 
   conflicting = conflictingBlock.Header
   
   if trusted.ValidatorsHash != conflicting.ValidatorsHash or
      trusted.NextValidatorsHash != conflicting.NextValidatorsHash or 
      trusted.ConsensusHash != conflicting.ConsensusHash or 
      trusted.AppHash != conflicting.AppHash or 
      trusted.LastResultsHash != conflicting.LastResultsHash {
        // find validators that have signed this header and that were present in trusted valset
        for (i, commitSig) in conflicting.Commit.Signatures {
            if commitSig.BlockIDFlag == BlockIDFlagCommit and 
            // TODO: think about this condition!   
            commitSig.ValidatorAddress in getValidators(bc[height].ValidatorsHash) {
                    evidence = LunaticValidatorEvidence { conflicting, createVote(commit, commitSig, i) }
                    evidences.append(evidence)
            }     
        }
   }
   return evidences         
} 

type Vote struct {
	Type             byte
	Height           int64
	Round            int
	BlockID          BlockID
	Timestamp        Time
	ValidatorAddress []byte
	ValidatorIndex   int
	Signature        []byte
}

func createVote(commit Commit, commitSig CommitSig, validatorIndex int) Vote {
    return Vote { 
                Type: precommit,
                Height: commit.Height,
                Round: commit.Round,            
                BlockID: commit.BlockID,
                Timestamp: commitSig.Timestamp,
                ValidatorAddress: commitSig.ValidatorAddress,
                ValidatorIndex: validatorIndex,
                Signature: commitSig.Signature                                      
           }
}

func createVote(commit Commit, validatorAddress ValidatorAddress) Vote {
    commitSig = nil
    for (i, commitSig) in commit.Signatures {
        if commitSig.validatorAddress == validatorAddress {
            return Vote { 
                            Type: precommit,
                            Height: commit.Height,
                            Round: commit.Round,            
                            BlockID: commit.BlockID,
                            Timestamp: commitSig.Timestamp,
                            ValidatorAddress: commitSig.ValidatorAddress,
                            ValidatorIndex: i,
                            Signature: commitSig.Signature                                      
                       }
        }
    }
    panic
}  
```

```go
func createDuplicateVoteEvidences(conflictingBlock LightBlock, bc Blockchain) []DuplicateVoteEvidence {
   evidences = []DuplicateVoteEvidence
   trusted = bc[conflicting.Header.Height].Commit
   conflicting = conflictingBlock.Commit

   if trusted.Round == conflicting.Round {
        for (i, commitSig) in conflicting.Signatures {
            if commitSig.BlockIDFlag == BlockIDFlagCommit and 
               commitSig.ValidatorAddress in getValidators(trusted) {
                    evidence = DuplicateVoteEvidence { 
                                    createVote(commit, commitSig.ValidatorAddress)
                                    createVote(commit, commitSig, i) 
                               }
                    evidences.append(evidence)
            }     
        }     
   } 
   
   if trusted.ValidatorsHash != conflicting.ValidatorsHash or
      trusted.NextValidatorsHash != conflicting.NextValidatorsHash or 
      trusted.ConsensusHash != conflicting.ConsensusHash or 
      trusted.AppHash != conflicting.AppHash or 
      trusted.LastResultsHash != conflicting.LastResultsHash {
        // find validators that have signed this header and that were present in trusted valset
        for (i, commitSig) in conflicting.Commit.Signatures {
            if commitSig.BlockIDFlag == BlockIDFlagCommit {
                evidence = LunaticValidatorEvidence { conflicting, createVote(commit, commitSig, i) }
                evidences.append(evidence)
            }     
        }
   }
   return evidences         
}



### Figuring out if malicious behaviour

The node first examines the case of a lunatic attack:

* The validator set of the common header must have at least 1/3 validator power that signed in the divergedHeaders commit

* One of the deterministically derived hashes (`ValidatorsHash`, `NextValidatorsHash`, `ConsensusHash`,
`AppHash`, or `LastResultsHash`) of the header must not match:

* We then take every validator that voted for the invalid header and was a validator in the common headers validator set and create `LunaticValidatorEvidence`

If this fails then we examine the case of Equivocation (either duplicate vote or amnesia):

*This only requires the trustedHeader and the divergedHeader*

* if `trustedHeader.Round == divergedHeader.Round`, and a validator signed for the block in both headers then DuplicateVoteEvidence can be immediately formed

* if `trustedHeader.Round != divergedHeader.Round` then we form PotentialAmnesiaEvidence as some validators in this set have behaved maliciously and protocol in ADR 56 needs to be run. 

*The node does not check that there is a 1/3 overlap between headers as this may not be point of the fork and validator sets may have since changed*

If no evidence can be formed from a light trace, it is not a legitimate trace and thus the 
connection with the peer should be stopped

### F1. Equivocation

Existing `DuplicateVoteEvidence` needs to be created and gossiped.


### F5. Lunatic validator



To punish this attack, we need support for a new Evidence type -
`LunaticValidatorEvidence`. This type includes a vote and a header. The header
must contain fields that are invalid with respect to the previous block, and a
vote for that header by a validator that was in a validator set within the
unbonding period. While the attack is only possible if +1/3 of some validator
set colludes, the evidence should be verifiable independently for each
individual validator. This means the total evidence can be split into one piece
of evidence per attacking validator and gossipped to nodes to be verified one
piece at a time, reducing the DoS attack surface at the peer layer.

Note it is not sufficient to simply compare this header with that committed for
the corresponding height, as an honest node may vote for a header that is not
ultimately committed. Certain fields may also be variable, for instance the
`LastCommitHash` and the `Time` may depend on which votes the proposer includes.
Thus, the header must be explicitly checked for invalid data.

For the attack to succeed, VC must sign a header that changes the validator set
to consist of something they control. Without doing this, they can not
otherwise attack the light client, since the client verifies commits according
to validator sets. Thus, it should be sufficient to check only that
`ValidatorsHash` and `NextValidatorsHash` are correct with respect to the
header that was committed at the corresponding height.

That said, if the attack is conducted by +2/3 of the validator set, they don't
need to make an invalid change to the validator set, since they already control
it. Instead they would make invalid changes to the `AppHash`, or possibly other
fields. In order to punish them, then, we would have to check all header
fields.

Note some header fields require the block itself to verify, which the light
client, by definition, does not possess, so it may not be possible to check
these fields. For now, then, `LunaticValidatorEvidence` must be checked against
all header fields which are a function of the application at previous blocks.
This includes `ValidatorsHash`, `NextValidatorsHash`, `ConsensusHash`,
`AppHash`, and `LastResultsHash`. These should all match what's in the header
for the block that was actually committed at the corresponding height, and
should thus be easy to check.

`InvalidHeaderField` contains the invalid field name. Note it's very likely
that multiple fields diverge, but it's faster to check just one. This field
MUST NOT be used to determine equality of `LunaticValidatorEvidence`.

### F2. Amnesia

```go
type PotentialAmnesiaEvidence struct {
	VoteA types.Vote
	VoteB types.Vote
}
```

To punish this attack, votes under question needs to be sent. Fork
accountability process should then use this evidence to request additional
information from offended validators and construct a new type of evidence to
punish those who conducted an amnesia attack.

See ADR-056 for the architecture of the handling amnesia attacks.

NOTE: Conflicting headers trace used to also create PhantomValidatorEvidence
but this has since been removed. Refer to Appendix B.  





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
**TODO:** finalize what this should do, and what detail of
  specification we need.
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
    - download block and compare to previously downloaded one.
- Expected precondition
- Expected postcondition
- Error condition


#### **[LCD-FUNC-REPLACE-PRIMARY.1]:**
```go
Replace_Primary()
```
**TODO:** formalize conditions
- Implementation remark
    - the primary is replaced by a secondary, and lightblocks above
      trusted blocks are removed
	- to maintain a constant size of secondaries, at this point we
      might need to 
	     - pick a new secondary *nsec*
		 - maintain [LCD-INV-TRUSTED-AGREED.1], that is,
		    - call `CrossCheck(nsec,lightStore.LatestTrusted()`.
              If it matches we are OK, otherwise
			     - we repeat with another full node as new
                   secondary candidate
				 - **FUTURE:** try to do fork detection from some possibly old
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


#### **[LCD-FUNC-REPLACE-SECONDARY.1]:**
```go
Replace_Secondary(addr Address)
```
**TODO:** formalize conditions
- Implementation remark
     - maintain [LCD-INV-TRUSTED-AGREED.1], that is,
		 - call `CrossCheck(nsec,lightStore.LatestTrusted()`.
           If it matches we are OK, otherwise
		   - we might just repeat with another full node as new secondary
		   - **FUTURE:** try to do fork detection from some possibly old
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
**TODO:** formalize conditions
- Expected precondition
	- Secondaries initialized and non-empty
	- `PoFs` initialized and empty
	- *lightStore.LatestTrusted().Height < lightStore.LatestVerified().Height*
- Expected postcondition
    - satisfies [LCD-DIST-INV.1], [LCD-DIST-LIFE-FORK.1]
	- removes faulty secondary if it reports wrong header
	- **TODO** submit proof of fork
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


[block]: https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md

[blockchain]: https://github.com/informalsystems/VDD/tree/master/blockchain/blockchain.md

[lightclient]: https://github.com/interchainio/tendermint-rs/blob/e2cb9aca0b95430fca2eac154edddc9588038982/docs/architecture/adr-002-lite-client.md

[verificationVDD]: https://github.com/informalsystems/VDD/blob/master/lightclient/failuredetector.md

[verification]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md

[accountability]: https://github.com/tendermint/spec/blob/master/spec/consensus/light-client/accountability.md

[tendermintfork]: https://docs.google.com/document/d/1xjyp5JOPt7QfHem1AFEaowBH2Plk0IHACWtYFXFvO7E/edit#heading=h.th2369ptc2ve
