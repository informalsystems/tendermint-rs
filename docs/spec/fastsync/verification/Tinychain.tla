-------------------------- MODULE Tinychain ----------------------------------
(* A very abstract model of Tendermint blockchain. Its only purpose is to highlight
   the relation between validator sets, next validator sets, and last commits.
 *)

EXTENDS Sequences, Integers

------------------------- MODULE ApalacheTypes ----------------------------------- 
\* type annotation
a <: b == a

\* the type of validator sets, e.g., STRING
VST == STRING

\* LastCommit type.
\* It contains the id of the committed block
\* and the set of the validators who have committed the block
LCT == [blockId |-> Int, commiters |-> VST]

\* Block type.
\* A block contains its height, validator set, next validator set, and last commit
BT == [height |-> Int, VS |-> VST, NextVS |-> VST, lastCommit |-> LCT]

SeqOfBT(s) == s <: Seq(BT)
==================================================================================

INSTANCE ApalacheTypes

CONSTANTS
    (*
       A set of abstract values, each value representing a set of validators.
       For the purposes of this specification, they can be any values,
       e.g., "s1", "s2", etc.
     *)
    VALIDATOR_SETS,
    (* The maximal height, up to which the blockchain may grow *)
    MAX_HEIGHT

VARIABLES
    (* The blockchain as a sequence of blocks *)
    chain

\* Initially, the blockchain contains one trusted block
Init ==
    \E vs \in VALIDATOR_SETS:
        \* the last commit in the trusted block is somewhat weird
        LET lastCommit == [blockId |-> 0, commiters |-> vs] IN 
        chain = SeqOfBT(<<
                    [height |-> 1, VS |-> vs, NextVS |-> vs, lastCommit |-> lastCommit
                ]>>)

\* Add one more block to the blockchain
AdvanceChain ==                
  \E NextVS \in VALIDATOR_SETS:
    LET last == chain[Len(chain)]
        lastCommit == [blockId |-> last.height, commiters |-> last.VS]
        newBlock == [height |-> last.height + 1,
                     VS |-> last.NextVS,
                     NextVS |-> NextVS,
                     lastCommit |-> lastCommit]
    IN
    chain' = Append(chain, newBlock)

\* The blockchain may grow up to the maximum and then stutter
Next ==
    \/ Len(chain) < MAX_HEIGHT /\ AdvanceChain
    \/ Len(chain) >= MAX_HEIGHT /\ UNCHANGED chain
    
\* The basic properties of blocks in the blockchain:
\* They should pass the validity check and they may verify the next block.    

\* Is a block valid against the next validator set (of the previous block)
IsValid(block, nextVS) ==
    \* simply check that the validator set is propagated correctly.
    \* (the implementation tests hashes and the application state)
    block.VS = nextVS

\* Does the block verify the commit (of the next block)
DoesVerify(block, commit) ==
    \* the commits are signed by the block validators
    /\ commit.commiters = block.VS
    \* the block id in the commit matches the block height
    \* (the implementation has more extensive tests)
    /\ commit.blockId = block.height


\* Basic invariants

\* every block has the validator set that is chosen by its predecessor
ValidBlockInv ==
    \A h \in 2..Len(chain):
        IsValid(chain[h], chain[h - 1].NextVS)

\* last commit of every block is signed by the validators of the predecessor     
VerifiedBlockInv ==
    \A h \in 2..Len(chain):
        DoesVerify(chain[h - 1], chain[h].lastCommit)

==================================================================================
