------------------------ MODULE Blockchain_A_1 -----------------------------
(* This is a high-level specification of Tendermint blockchain
   that is designed specifically for:
   
   (1) Lite client, and
   (2) Fork accountability.
 *)
EXTENDS Integers, Sequences

Min(a, b) == IF a < b THEN a ELSE b

CONSTANT
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  ULTIMATE_HEIGHT,
    (* a maximal height that can be ever reached (modelling artifact) *)
  MAX_POWER
    (* a maximal voting power of a single node *)

Heights == 0..ULTIMATE_HEIGHT   (* possible heights *)

Powers == 1..MAX_POWER          (* possible voting powers *)

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
  VP: UNION {[Nodes -> Powers]: Nodes \in SUBSET AllNodes \ {{}}},
    \* the validators of this block together with their voting powers,
    \* i.e., a multi-set. We store the validators instead of the hash.
  NextVP: UNION {[Nodes -> Powers]: Nodes \in SUBSET AllNodes \ {{}}}
    \* the validators of the next block together with their voting powers,
    \* i.e., a multi-set. We store the next validators instead of the hash.
]

(* A convenience operator that retrieves the validator set from a header *)
VS(header) == DOMAIN header.VP

(* A convenience operator that retrieves the next validator set from a header *)
NextVS(header) == DOMAIN header.NextVP

(* A signed header is just a header together with a set of commits *)
\* TODO: Commits is the set of PRECOMMIT messages
SignedHeaders == [header: BlockHeaders, Commits: Commits]

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

(****************************** BLOCKCHAIN ************************************)
(* in the future, we may extract it in a module on its own *)

(*
Compute the total voting power of a subset of pNodes \subseteq AllNodes,
whose individual voting powers are given with a function
pVotingPower \in AllNodes -> Powers.
*)  
RECURSIVE PowerOfSet(_, _)
PowerOfSet(pVotingPower, pNodes) ==
    IF pNodes = {}
    THEN 0
    ELSE LET node == CHOOSE n \in pNodes: TRUE IN
        (* compute the voting power for the nodes in Nodes \ {node}
           and sum it up with the node's power *)
        pVotingPower[node] + PowerOfSet(pVotingPower, pNodes \ {node})

(*
 Given a function pVotingPower \in D -> Powers for some D \subseteq AllNodes
 and pNodes \subseteq D, test whether the set pNodes \subseteq AllNodes has
 more than 2/3 of voting power among the nodes in D.
 *)
TwoThirds(pVotingPower, pNodes) ==
    LET TP == PowerOfSet(pVotingPower, DOMAIN pVotingPower)
        SP == PowerOfSet(pVotingPower, pNodes)
    IN
    3 * SP > 2 * TP \* when thinking in real numbers, not integers: SP > 2.0 / 3.0 * TP 

(*
 Given a function pVotingPower \in D -> Powers for some D \subseteq pNodes,
 and a set of pFaultyNodes, test whether the voting power of the correct
 nodes in pNodes is more than 2/3 of the voting power of the faulty nodes
 among the nodes in pNodes.
 *)
IsCorrectPowerForSet(pFaultyNodes, pVotingPower, pNodes) ==
    LET FN == pFaultyNodes \intersect pNodes  \* faulty nodes in pNodes
        CN == pNodes \ pFaultyNodes           \* correct nodes in pNodes
        CP == PowerOfSet(pVotingPower, CN)   \* power of the correct nodes
        FP == PowerOfSet(pVotingPower, FN)   \* power of the faulty nodes
    IN
    \* CP + FP = TP is the total voting power, so we write CP > 2.0 / 3 * TP as follows:
    CP > 2 * FP \* Note: when FP = 0, this implies CP > 0.

(*
 Given a function votingPower \in D -> Power for some D \subseteq Nodes,
 and a set of FaultyNodes, test whether the voting power of the correct nodes in D
 is more than 2/3 of the voting power of the faulty nodes in D.
 *)
IsCorrectPower(pFaultyNodes, pVotingPower) ==
    IsCorrectPowerForSet(pFaultyNodes, pVotingPower, DOMAIN pVotingPower)
    
(* This is what we believe is the assumption about failures in Tendermint *)     
FaultAssumption(pFaultyNodes, pMinTrustedHeight, pBlockchain) ==
    \A h \in pMinTrustedHeight..Len(pBlockchain):
        IsCorrectPower(pFaultyNodes, pBlockchain[h].NextVP)


(* A signed header whose commit coincides with the last commit of a block,
   unless the commits are made by the faulty nodes *)
SoundSignedHeaders(ht) ==
    { sh \in SignedHeaders:
        \/ sh.header = blockchain[ht] \* signed by correct and faulty (maybe)
        \/ sh.Commits \subseteq Faulty /\ sh.header.height = ht \* signed only by faulty
    }


(* Append a new block on the blockchain.
   Importantly, more than 2/3 of voting power in the next set of validators
   belongs to the correct processes. *)       
AppendBlock ==
  LET last == blockchain[Len(blockchain)] IN
  \E lastCommit \in SUBSET (VS(last)) \ {{}},
     NextV \in SUBSET AllNodes \ {{}}:
     \E NextVP \in [NextV -> Powers]:
    LET new == [ height |-> height + 1, lastCommit |-> lastCommit,
                 VP |-> last.NextVP, NextVP |-> NextVP ] IN
    /\ TwoThirds(last.VP, lastCommit)
    /\ IsCorrectPower(Faulty, NextVP) \* the correct validators have >2/3 of power
    /\ blockchain' = Append(blockchain, new)
    /\ height' = height + 1

(* Initialize the blockchain *)
Init ==
  /\ height = 1             \* there is just genesis block
  /\ minTrustedHeight = 1   \* the genesis is initially trusted
  /\ Faulty = {}            \* initially, there are no faults
  /\ tooManyFaults = FALSE  \* there are no faults
  (* pick a genesis block of all nodes where next correct validators have >2/3 of power *)
  /\ \E NextV \in SUBSET AllNodes:          \* pick a next validator set
       \E NextVP \in [NextV -> Powers]:     \* and pick voting powers to every node
         LET VP == [n \in AllNodes |-> 1]
             \* assume that the validators of the genesis have voting power of 1
             \* and construct the genesis block
             genesis == [ height |-> 1, lastCommit |-> {},
                          VP |-> VP, NextVP |-> NextVP]
         IN
         /\ NextV /= {}     \* assume that there is at least one next validator 
         /\ blockchain = <<genesis>> \* initially, blockchain contains only the genesis

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
      NextVS(blockchain[h]) /= {}

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
\* Last modified Tue Nov 19 11:15:32 CET 2019 by igor
\* Created Fri Oct 11 15:45:11 CEST 2019 by igor
