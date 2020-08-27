
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




 


