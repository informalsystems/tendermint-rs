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
  ULTIMATE_HEIGHT,
    (* a maximal height that can be ever reached (modelling artifact) *)
  TRUSTING_PERIOD
    (* the period within which the validators are trusted *)

Heights == 1..ULTIMATE_HEIGHT   (* possible heights *)

(* A commit is just a set of nodes who have committed the block *)
Commits == SUBSET AllNodes

(* The set of all block headers that can be on the blockchain.
   This is a simplified version of the Block data structure in the actual implementation. *)
BlockHeaders == [
  height: Heights,
    \* the block height
  time: Int,
    \* the block timestamp in some integer units
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
    \*tooManyFaults,
    (* whether there are more faults in the system than the blockchain can handle *)
    height,
    (* the height of the blockchain, starting with 0 *)
    \*minTrustedHeight,
    (* The global height of the oldest block that is younger than
       the trusted period (AKA the almost rotten block).
       In the implementation, this is the oldest block,
       where block.bftTime + trustingPeriod >= globalClock.now. *)
    now,
        (* the current global time in integer units *)
    blockchain,
    (* A sequence of BlockHeaders, which gives us a bird view of the blockchain. *)
    Faulty
    (* A set of faulty nodes, which can act as validators. We assume that the set
       of faulty processes is non-decreasing. If a process has recovered, it should
       connect using a different id. *)
       
(* all variables, to be used with UNCHANGED *)       
vars == <<(*tooManyFaults,*) height, (*minTrustedHeight,*) now, blockchain, Faulty>>         

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

(* the header is still within the trusting period *)
InTrustingPeriod(header) ==
    now <= header.time + TRUSTING_PERIOD

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
FaultAssumption(pFaultyNodes, (*pMinTrustedHeight,*) pNow, pBlockchain) ==
    \A h \in Heights:
      (*pMinTrustedHeight <= h*)
      pBlockchain[h].time + TRUSTING_PERIOD > pNow =>
        IsCorrectPower(pFaultyNodes, pBlockchain[h].NextVS)

(* Can a block be produced by a correct peer, or an authenticated Byzantine peer *)
IsProducableByFaulty(ht, block) == 
    \/ block.header = blockchain[ht] \* signed by correct and faulty (maybe)
    \/ block.Commits \subseteq Faulty /\ block.header.height = ht \* signed only by faulty

(* A light block whose commit coincides with the last commit of a block,
   unless the commits are made by the faulty nodes *)
ProducableByFaultyLightBlocks(ht) ==
    { b \in LightBlocks: IsProducableByFaulty(ht, b) }

(*
 Initialize the blockchain to the ultimate height right in the initial states.
 We pick the faulty validators statically, but that should not affect the light client.
 *)            
InitToHeight ==
  /\ height = ULTIMATE_HEIGHT
  \*/\ minTrustedHeight = 1       \* all blocks are initially trusted
  /\ Faulty \in SUBSET AllNodes \* some nodes may fail
  \*/\ tooManyFaults = FALSE      \* we pick blocks so the blockchain is in the green zone
  \* pick the validator sets and last commits
  /\ \E vs, lastCommit \in [Heights -> SUBSET AllNodes]:
     \E timestamp \in [Heights -> Int]:
        \* now is at least as early as the timestamp in the last block 
        /\ \E tm \in Int: now = tm /\ tm >= timestamp[ULTIMATE_HEIGHT]
        \* the genesis starts on day 1     
        /\ timestamp[1] = 1
        /\ vs[1] = AllNodes
        /\ lastCommit[1] = EmptyNodeSet
        /\ \A h \in Heights \ {1}:
          /\ lastCommit[h] \subseteq vs[h - 1]   \* the non-validators cannot commit 
          /\ TwoThirds(vs[h - 1], lastCommit[h]) \* the commit has >2/3 of validator votes
          /\ IsCorrectPower(Faulty, vs[h])       \* the correct validators have >2/3 of power
          /\ timestamp[h] >= timestamp[h - 1]    \* the time grows monotonically
          /\ timestamp[h] < timestamp[h - 1] + TRUSTING_PERIOD    \* but not too fast
        \* form the block chain out of validator sets and commits (this makes apalache faster)
        /\ blockchain = [h \in Heights |->
             [height |-> h,
              time |-> timestamp[h],
              VS |-> vs[h],
              NextVS |-> IF h < ULTIMATE_HEIGHT THEN vs[h + 1] ELSE AllNodes,
              lastCommit |-> lastCommit[h]]
             ] \******
       

(* is the blockchain in the faulty zone where the Tendermint security model does not apply *)
InFaultyZone ==
  ~FaultAssumption(Faulty, (*minTrustedHeight,*) now, blockchain)       

(********************* BLOCKCHAIN ACTIONS ********************************)
(*
  Advance the clock by zero or more time units.
  *)
AdvanceTime ==
  \E tm \in Int:
    /\ tm >= now
    /\ now' = tm

  \*/\ minTrustedHeight' \in (minTrustedHeight + 1) .. Min(height + 1, ULTIMATE_HEIGHT)
  \* we are using IF-THEN-ELSE, otherwise Apalache may produce a spurious counterexample
  \* https://github.com/konnov/apalache/issues/148
  (*
  /\ IF FaultAssumption(Faulty, minTrustedHeight', blockchain)
     THEN tooManyFaults' = FALSE
     ELSE tooManyFaults' = TRUE
   *) 
  /\ UNCHANGED <<height, blockchain, Faulty>>

(*
 One more process fails. As a result, the blockchain may move into the faulty zone.
 The light client is not using this action, as the faults are picked in the initial state.
 However, this action may be useful when reasoning about fork detection.
 *)
OneMoreFault ==
  /\ \E n \in AllNodes \ Faulty:
      /\ Faulty' = Faulty \cup {n}
      /\ Faulty' /= AllNodes \* at least process remains non-faulty
      \* we are using IF-THEN-ELSE, otherwise Apalache may produce a spurious counterexample
      (*
      /\ IF FaultAssumption(Faulty', minTrustedHeight, blockchain)
         THEN tooManyFaults' = FALSE
         ELSE tooManyFaults' = TRUE
       *) 
  /\ UNCHANGED <<height, (*minTrustedHeight,*) now, blockchain>>

(* stuttering at the end of the blockchain *)
StutterInTheEnd == 
  height = ULTIMATE_HEIGHT /\ UNCHANGED vars

(********************* PROPERTIES TO CHECK ********************************)
(* False properties that can be checked with TLC, to see interesting behaviors *)

(* Check this to see how the blockchain can jump into the faulty zone *)
\*NeverFaulty == ~tooManyFaults

(* check this to see how the trusted period can expire *)
\*NeverUltimateHeight ==
\*  minTrustedHeight < ULTIMATE_HEIGHT

=============================================================================
\* Modification History
\* Last modified Fri Jun 05 13:11:42 CEST 2020 by igor
\* Created Fri Oct 11 15:45:11 CEST 2019 by igor
