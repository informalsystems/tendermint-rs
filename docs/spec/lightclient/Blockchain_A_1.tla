------------------------ MODULE Blockchain_A_1 -----------------------------
(*
  This is a high-level specification of Tendermint blockchain
  that is designed specifically for the light client.
  Validators have the voting power of one. If you like to model various
  voting powers, introduce multiple copies of the same validator
  (do not forget to give them unique names though).
 *)
EXTENDS Integers, Sequences, FiniteSets

Min(a, b) == IF a < b THEN a ELSE b

CONSTANT
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  ULTIMATE_HEIGHT
    (* a maximal height that can be ever reached (modelling artifact) *)

Heights == 0..ULTIMATE_HEIGHT   (* possible heights *)

(* A commit is just a set of nodes who have committed the block *)
Commits == SUBSET AllNodes

(* The set of all block headers that can be on the blockchain.
   This is a simplified version of the Block data structure in the actual implementation. *)
BlockHeaders == [
  height: Heights,
    \* the block height
  lastCommit: Commits,
    \* the nodes who have voted on the previous block, the set itself instead of a hash
  (* in the implementation, only the hashes of V and NextV are stored in a block,
     as V and NextV are stored in the application state *) 
  VS: SUBSET AllNodes,
    \* the validators of this bloc. We store the validators instead of the hash.
  NextVS: SUBSET AllNodes
    \* the validators of the next block. We store the next validators instead of the hash.
]

(* A signed header is just a header together with a set of commits *)
LightBlocks == [header: BlockHeaders, Commits: Commits]

VARIABLES
    tooManyFaults,
    (* whether there are more faults in the system than the blockchain can handle *)
    height,
    (* the height of the blockchain, starting with 0 *)
    minTrustedHeight,
    (* The global height of the oldest block that is younger than
       the trusted period (AKA the almost rotten block).
       In the implementation, this is the oldest block,
       where block.bftTime + trustingPeriod >= globalClock.now. *)
    blockchain,
    (* A sequence of BlockHeaders, which gives us a bird view of the blockchain. *)
    Faulty
    (* A set of faulty nodes, which can act as validators. We assume that the set
       of faulty processes is non-decreasing. If a process has recovered, it should
       connect using a different id. *)
       
(* all variables, to be used with UNCHANGED *)       
vars == <<tooManyFaults, height, minTrustedHeight, blockchain, Faulty>>         

(* The set of all correct nodes in a state *)
Corr == AllNodes \ Faulty

(* APALACHE annotations *)
a <: b == a \* type annotation

NT == STRING
NodeSet(S) == S <: {NT}
EmptyNodeSet == NodeSet({})

BT == [height |-> Int, lastCommit |-> {NT}, VS |-> {NT}, NextVS |-> {NT}]
BlockSeq(seq) == seq <: Seq(BT)        

LBT == [header |-> BT, Commits |-> {NT}]
(* end of APALACHE annotations *)       

(****************************** BLOCKCHAIN ************************************)

(*
 Given a function pVotingPower \in D -> Powers for some D \subseteq AllNodes
 and pNodes \subseteq D, test whether the set pNodes \subseteq AllNodes has
 more than 2/3 of voting power among the nodes in D.
 *)
TwoThirds(pVS, pNodes) ==
    LET TP == Cardinality(pVS)
        SP == Cardinality(pVS \intersect pNodes)
    IN
    3 * SP > 2 * TP \* when thinking in real numbers, not integers: SP > 2.0 / 3.0 * TP 

(*
 Given a function pVotingPower \in D -> Powers for some D \subseteq pNodes,
 and a set of pFaultyNodes, test whether the voting power of the correct
 nodes in pNodes is more than 2/3 of the voting power of the faulty nodes
 among the nodes in pNodes.
 *)
IsCorrectPowerForSet(pFaultyNodes, pVS) ==
    LET FN == pFaultyNodes \intersect pVS   \* faulty nodes in pNodes
        CN == pVS \ pFaultyNodes            \* correct nodes in pNodes
        CP == Cardinality(CN)               \* power of the correct nodes
        FP == Cardinality(FN)               \* power of the faulty nodes
    IN
    \* CP + FP = TP is the total voting power, so we write CP > 2.0 / 3 * TP as follows:
    CP > 2 * FP \* Note: when FP = 0, this implies CP > 0.

(*
 Given a function votingPower \in D -> Power for some D \subseteq Nodes,
 and a set of FaultyNodes, test whether the voting power of the correct nodes in D
 is more than 2/3 of the voting power of the faulty nodes in D.
 *)
IsCorrectPower(pFaultyNodes, pVS) ==
    IsCorrectPowerForSet(pFaultyNodes, pVS)
    
(* This is what we believe is the assumption about failures in Tendermint *)     
FaultAssumption(pFaultyNodes, pMinTrustedHeight, pBlockchain) ==
    \A h \in pMinTrustedHeight..Len(pBlockchain):
        IsCorrectPower(pFaultyNodes, pBlockchain[h].NextVS)

(* Can a block be produced by a correct peer, or an authenticated Byzantine peer *)
IsProducableByFaulty(ht, block) == 
    \/ block.header = blockchain[ht] \* signed by correct and faulty (maybe)
    \/ block.Commits \subseteq Faulty /\ block.header.height = ht \* signed only by faulty

(* A light block whose commit coincides with the last commit of a block,
   unless the commits are made by the faulty nodes *)
ProducableByFaultyLightBlocks(ht) ==
    { b \in LightBlocks: IsProducableByFaulty(ht, b) }

(* Append a new block on the blockchain.
   Importantly, more than 2/3 of voting power in the next set of validators
   belongs to the correct processes. *)       
AppendBlock ==
  LET last == blockchain[Len(blockchain)] IN
  \E lastCommit \in SUBSET (last.VS),
     NVS \in SUBSET AllNodes:
    /\ lastCommit /= EmptyNodeSet
    /\ NVS /= EmptyNodeSet
    /\ LET new == [ height |-> height + 1, lastCommit |-> lastCommit,
                    VS |-> last.NextVS, NextVS |-> NVS ] IN
       /\ TwoThirds(last.VS, lastCommit)
       /\ IsCorrectPower(Faulty, NVS) \* the correct validators have >2/3 of power
       /\ blockchain' = Append(blockchain, new)
       /\ height' = height + 1

(* Initialize the blockchain *)
Init ==
  /\ height = 1             \* there is just genesis block
  /\ minTrustedHeight = 1   \* the genesis is initially trusted
  /\ Faulty = EmptyNodeSet  \* initially, there are no faults
  /\ tooManyFaults = FALSE  \* there are no faults
  (* pick a genesis block of all nodes where next correct validators have >2/3 of power *)
  /\ \E NVS \in SUBSET AllNodes:          \* pick a next validator set
         /\ NVS /= EmptyNodeSet     \* assume that there is at least one next validator 
         /\ LET genesis ==
              [ height |-> 1, lastCommit |-> EmptyNodeSet, VS |-> AllNodes, NextVS |-> NVS]
            IN
             \* initially, blockchain contains only the genesis
            blockchain = BlockSeq(<<genesis>>)

(********************* BLOCKCHAIN ACTIONS ********************************)
          
(*
  The blockchain may progress by adding one more block, provided that:
     (1) The ultimate height has not been reached yet, and
     (2) The faults are within the bounds.
 *)
AdvanceChain ==
  /\ height < ULTIMATE_HEIGHT /\ ~tooManyFaults
  /\ AppendBlock
  /\ UNCHANGED <<minTrustedHeight, tooManyFaults, Faulty>>

(*
  As time is passing, the minimal trusted height may increase.
  As a result, the blockchain may move out of the faulty zone.
  *)
AdvanceTime ==
  /\ minTrustedHeight' \in (minTrustedHeight + 1) .. Min(height + 1, ULTIMATE_HEIGHT)
  /\ tooManyFaults' = ~FaultAssumption(Faulty, minTrustedHeight', blockchain)
  /\ UNCHANGED <<height, blockchain, Faulty>>

(* One more process fails. As a result, the blockchain may move into the faulty zone. *)
OneMoreFault ==
  /\ \E n \in AllNodes \ Faulty:
      /\ Faulty' = Faulty \cup {n}
      /\ Faulty' /= AllNodes \* at least process remains non-faulty
      /\ tooManyFaults' = ~FaultAssumption(Faulty', minTrustedHeight, blockchain)
  /\ UNCHANGED <<height, minTrustedHeight, blockchain>>

(* stuttering at the end of the blockchain *)
StutterInTheEnd == 
  height = ULTIMATE_HEIGHT /\ UNCHANGED vars

(* Let the blockchain to make progress *)
Next ==
  \/ AdvanceChain
  \/ AdvanceTime
  \/ OneMoreFault
  \/ StutterInTheEnd

(********************* PROPERTIES TO CHECK ********************************)

(* Invariant: it should be always possible to add one more block unless:
  (1) either there are too many faults,
  (2) or the bonding period of the last transaction has expired, so we are stuck.
 *)
NeverStuck ==
  \/ tooManyFaults
  \/ height = ULTIMATE_HEIGHT
  \/ minTrustedHeight > height \* the trusting period has expired
  \/ ENABLED AdvanceChain

(* The next validator set is never empty *)
NextVSNonEmpty ==
    \A h \in 1..Len(blockchain):
      blockchain[h].NextVS /= EmptyNodeSet

(* False properties that can be checked with TLC, to see interesting behaviors *)

(* Check this to see how the blockchain can jump into the faulty zone *)
NeverFaulty == ~tooManyFaults

(* check this to see how the trusted period can expire *)
NeverUltimateHeight ==
  minTrustedHeight < ULTIMATE_HEIGHT

(* False: it should be always possible to add one more block *)
NeverStuckFalse1 ==
  ENABLED AdvanceChain

(* False: it should be always possible to add one more block *)
(*
   TODO: this property is not false anymore, it is possible to add a block after
   the trusted period has expired!
 *)
NeverStuckFalse2 ==
  \/ tooManyFaults
  \/ height = ULTIMATE_HEIGHT
  \/ ENABLED AdvanceChain

=============================================================================
\* Modification History
\* Last modified Wed Jun 03 10:06:10 CEST 2020 by igor
\* Created Fri Oct 11 15:45:11 CEST 2019 by igor
