-------------------------- MODULE Tinychain ----------------------------------
(* A very abstract model of Tendermint blockchain. Its only purpose is to highlight
   the relation between validator sets, next validator sets, and last commits.
 *)

EXTENDS Sequences, Integers

\* type annotation
a <: b == a

\* the type of validator sets, e.g., STRING
VST == STRING

\* LastCommit type.
\* It contains the id of the committed block
\* and the set of the validators who have committed the block.
\* In the implementation, blockId is the hash of the previous block, which cannot be forged
LCT == [blockId |-> Int, commiters |-> VST]

\* Block type.
\* A block contains its height, validator set, next validator set, and last commit
BT == [height |-> Int, hash |-> Int, wellFormed |-> BOOLEAN,
       VS |-> VST, NextVS |-> VST, lastCommit |-> LCT]

SeqOfBT(s) == s <: Seq(BT)


CONSTANTS
    (*
       A set of abstract values, each value representing a set of validators.
       For the purposes of this specification, they can be any values,
       e.g., "s1", "s2", etc.
     *)
    VALIDATOR_SETS,
    (* a nil validator set that is outside of VALIDATOR_SETS *)
    NIL_VS,
    (* The maximal height, up to which the blockchain may grow *)
    MAX_HEIGHT

VARIABLES
    (* The blockchain as a sequence of blocks *)
    chain
    
Heights == 1..MAX_HEIGHT    

\* the set of block identifiers, simply the heights extended with 0
BlockIds == 0..MAX_HEIGHT

\* the set of all possible commits
Commits == [blockId: BlockIds, commiters: VALIDATOR_SETS]

\* the set of all possible blocks, not necessarily valid ones
Blocks ==
  [height: Heights, hash: Heights, wellFormed: BOOLEAN,
   VS: VALIDATOR_SETS, NextVS: VALIDATOR_SETS, lastCommit: Commits]

\* Initially, the blockchain contains one trusted block
ChainInit ==
    \E vs \in VALIDATOR_SETS:
        \* the last commit in the trusted block is somewhat weird
        LET lastCommit == [blockId |-> 0, commiters |-> vs] IN 
        chain = SeqOfBT(<<
                    [height |-> 1, hash |-> 1, wellFormed |-> TRUE,
                     VS |-> vs, NextVS |-> vs, lastCommit |-> lastCommit
                ]>>)

\* Add one more block to the blockchain
AdvanceChain ==                
  \E NextVS \in VALIDATOR_SETS:
    LET last == chain[Len(chain)]
        lastCommit == [blockId |-> last.hash, commiters |-> last.VS]
        newBlock == [height |-> last.height + 1,
                     hash |-> last.hash + 1,
                     VS |-> last.NextVS,
                     NextVS |-> NextVS,
                     lastCommit |-> lastCommit,
                     wellFormed |-> TRUE]
    IN
    chain' = Append(chain, newBlock)

\* The blockchain may grow up to the maximum and then stutter
ChainNext ==
    \/ Len(chain) < MAX_HEIGHT /\ AdvanceChain
    \/ Len(chain) >= MAX_HEIGHT /\ UNCHANGED chain
    
\* The basic properties of blocks in the blockchain:
\* They should pass the validity check and they may verify the next block.    

\* Does the block pass the consistency check against the next validators of the previous block
IsMatchingValidators(block, nextVS) ==
    \* simply check that the validator set is propagated correctly.
    \* (the implementation tests hashes and the application state)
    block.VS = nextVS

\* Does the block verify the commit (of the next block)
PossibleCommit(block, commit) ==
    \* the commits are signed by the block validators
    /\ commit.commiters = block.VS
    \* the block id in the commit matches the block hash
    \* (the implementation has more extensive tests)
    /\ commit.blockId = block.hash


\* Basic invariants

\* every block has the validator set that is chosen by its predecessor
ValidBlockInv ==
    \A h \in 2..Len(chain):
        IsMatchingValidators(chain[h], chain[h - 1].NextVS)

\* last commit of every block is signed by the validators of the predecessor     
VerifiedBlockInv ==
    \A h \in 2..Len(chain):
        PossibleCommit(chain[h - 1], chain[h].lastCommit)

==================================================================================
