
# Light client attacks

We define a light client attack as an observation of conflicting headers for a given height that can be verified
starting from the common light block. A light client attack is defined in the context of interactions of light client
with two peers. One of the peers (called primary) defines a trace of verified headers (primary trace) that are 
being checked against trace (witness trace) from the other peer (called witness). 

A light client attack is defined by the primary and witness traces 
that have a common root (the same header for a common height) but forms conflicting branches (end of traces is 
for the same height but with different headers). As conflicting branches could be arbitrarily big (
as branches continue do diverge after a bifurcation point), we define a valid light client attack
as one where a primary trace is of height two (consist of a common header and a conflicting header) and 
we don't constrain witness trace. The rational is the fact that we assume that the primary is under suspicion
(therefore not trusted) and the witness plays support role to detect and process an attack (therefore trusted).
Therefore, once a light client detect an attack, it needs to send to a witness only missing data (common height
and conflicting light block) as it has its trace. Keeping light client attack data of constant size saves bandwidth and 
reduces an attack surface. As we will explain below, although in the context of light client core verification
(TODO: add reference) the roles of primary and witness are clearly defined, in case of the attack, we run 
the same procedure attack detection procedure twice where the roles are swapped. The rationale is that 
light client does not know what peer is correct (on a right main branch) so it tries to create and submit
an attack evidence to both peers. 
   

Light client attack evidence consists of a conflicting light block, a common height and an attack type. There are 
three types of attacks that can be executed against Tendermint light client: 
    - lunatic attack
    - equivocation attack and 
    - amnesia attack. 

TODO: Add references to attack definitions.     
TODO: Add Callum's figures.
    
```go
type LightClientAttackEvidence struct {
    ConflictingBlock   LightBlock
    CommonHeight       int32
    Type               AttackType
}

enum AttackType {LunaticAttack, EquivocationAttack, AmnesiaAttack}
```

Full node can validate a light client attack by executing the following procedure:

```go
func IsValid(lcAttack LightClientAttackEvidence, bc Blockchain) boolean {
    commonBlock = GetLightBlock(bc, lcAttack.CommonHeight)
    if commonBlock == nil return false 
    
    // Note that trustingPeriod in ValidAndVerified is set to UNBONDING_PERIOD
    verdict = ValidAndVerified(commonBlock, lcAttack.ConflictingBlock)
    conflictingHeight = lcAttack.ConflictingBlock.Header.Height
    if verdict != OK or bc[conflictingHeight].Header == lcAttack.ConflictingBlock.Header {
        return false
    }
    
    trustedBlock = GetLightBlock(bc, conflictingHeight)
    switch lcAttack.Type {
        
        case LunaticAttack: return !isPossibleBlock(trustedBlock, lcAttack.ConflictingBlock)
        
        case EquivocationAttack: return isPossibleBlock(trustedBlock, lcAttack.ConflictingBlock) and 
                                        trustedBlock.Header.Round == lcAttack.ConflictingBlock.Header.Round

        case AmnesiaAttack: return isPossibleBlock(trustedBlock, lcAttack.ConflictingBlock) and 
                                   trustedBlock.Header.Round != lcAttack.ConflictingBlock.Header.Round
    } 
}

func isPossibleBlock(trusted Header, conflicting Header) boolean {
    return trusted.ValidatorsHash == conflicting.ValidatorsHash and
           trusted.NextValidatorsHash == conflicting.NextValidatorsHash and
           trusted.ConsensusHash == conflicting.ConsensusHash and 
           trusted.AppHash == conflicting.AppHash and 
           trusted.LastResultsHash == conflicting.LastResultsHash 
}
```


## Light client attack creation

```go
func DetectLightClientAttacks(primary PeerID, 
                        primary_trace []LightBlock, 
                        witness PeerID) (LightClientAttackEvidence, LightClientAttackEvidence) {
    primary_lca, witness_trace = DetectLightClientAttack(primary_trace, witness)
    
    witness_lca = nil
    if witness_trace != nil {
        witness_lca, _ = DetectLightClientAttack(witness_trace, primary)
    }
    return primary_lca, witness_lca
}

func DetectLightClientAttack(trace []LightBlock, peer PeerID) (LightClientAttackEvidence, []LightBlock) {

    trusted = trace[0]
    lightStore = new LightStore().Update(trusted, StateTrusted)

    for i in 1..len(trace)-1 {
        lightStore, result = VerifyToTarget(peer, lightStore , i)
   
        if result == ResultFailure then return (nil, nil)
        
        current = lightStore.Get(i)
        
        // if obtained header is the same as in the trace we continue with a next height
        if current.Header == trace[i].Header continue

        // we have identified a conflicting header
        attackType = nil
        trustedBlock = trace[i-1]
        conflictingBlock = trace[i]
        
        if !isPossibleBlock(trustedBlock, conflictingBlock) { 
            attackType = LunaticAttack
        } else if trustedBlock.Header.Round == conflictingBlock.Header.Round {
            attackType = EquivocationAttack
        } else {
           attackType = AmnesiaAttack
        }                                        
                 
        return (LightClientAttackEvidence { conflictingBlock, trustedBlock.Header.Height }, 
                lightStore.Trace(trace[i-1].Height, trace[i].Height))
    } 
    return (nil, nil)       
}
```

## Evidence handling 

As part of on chain evidence handling, full nodes identifies misbehaving processes and informs
the application, so they can be slashed. We now specify for each event type evidence handling logic.

### Lunatic attack evidence handling

```go
func detectMisbehavingProcesses(lcAttackEvidence LightClientAttackEvidence, bc Blockchain) []ValidatorAddress {
   assume lcAttackEvidence.Type == LunaticAttack
   misbehavingValidators = []ValidatorAddress

   conflictingHeight = lcAttackEvidence.ConflictingBlock.Header.Height
   
   bondedValidators = bc.GetValidators(lcAttackEvidence.CommonHeight) union 
                      bc.GetNextValidators(lcAttackEvidence.CommonHeight) union
                      bc.GetValidators(conflictingHeight)) 
   
   return getSigners(lcAttackEvidence.ConflictingBlock.Commit) intersection bondedValidators        
}

func getSigners(commit Commit) []ValidatorAddress {
    signers = []ValidatorAddress
    for (i, commitSig) in commit.Signatures {
        if commitSig.BlockIDFlag == BlockIDFlagCommit { 
            signers.append(commitSig.ValidatorAddress)                    
        }
    }
    return signers
}
```

### Equivocation attack evidence handling

```go
func detectMisbehavingProcesses(lcAttackEvidence LightClientAttackEvidence, bc Blockchain) []ValidatorAddress {
   assume lcAttackEvidence.Type == EquivocationAttack
   
   conflictingBlock = lcAttackEvidence.ConflictingBlock 
   trusted = bc[conflictingBlock.Header.Height+1].LastCommit 
   conflicting = conflictingBlock.Commit
   
   return getSigners(trusted) intersection getSigners(conflicting)      
}
```

### Amnesia attack evidence handling

Detecting faulty processes in case of the amnesia attack is more complex and cannot be inferred 
purely based on attack evidence data. In this case, in order to detect misbehaving processes we need
access to votes processes sent during the conflicting height. Therefore, amnesia handling assumes that
validators persist all votes received and sent during multi-round heights (as amnesia attack 
is only possible in heights that executes over multiple rounds, i.e., commit round > 0).  

To simplify description of the algorithm we assume existence of the trusted oracle called monitor that will 
drive the algorithm and output faulty processes at the end. Monitor can be implemented in a
distributed setting as on-chain module. The algorithm works as follows:
    1) Monitor sends votesets request to validators of the conflicting height. Validators
    are expected to send their votesets within predefined timeout.
    2) Upon receiving votesets request, validators send their votesets to a monitor.  
    2) Validators which have not sent its votesets within timeout are considered faulty.
    3) The preprocessing of the votesets is done. That means that the received votesets are analyzed 
    and each vote (valid) sent by process p is added to the voteset of the sender p. This phase ensures that
    votes sent by faulty processes observed by at least one correct validator cannot be excluded from the analysis. 
    4) Votesets of every validator is analyzed independently to decide whether the validator is correct or faulty.
       A faulty validators is the one where at least one of those invalid transitions is found:
            - More than one PREVOTE message is sent in a round 
            - More than one PRECOMMIT message is sent in a round 
            - PRECOMMIT message is sent without receiving +2/3 of voting-power equivalent 
            appropriate PREVOTE messages 
            - PREVOTE message is sent for the value V’ in round r’ and the PRECOMMIT message had 
            been sent for the value V in round r by the same process (r’ > r) and there are no 
            +2/3 of voting-power equivalent PREVOTE(vr, V’) messages (vr ≥ 0 and vr > r and vr < r’) 
            as the justification for sending PREVOTE(r’, V’) 




 


