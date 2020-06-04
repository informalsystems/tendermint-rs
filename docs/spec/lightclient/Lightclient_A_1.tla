-------------------------- MODULE Lightclient_A_1 ----------------------------
(*
 * A state-machine specification of the lite client, following the English spec:
 * https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md
 *) 

EXTENDS Integers, Sequences, FiniteSets

\* the parameters of Lite Client
CONSTANTS
  TRUSTED_HEIGHT,
    (* an index of the block header that the light client trusts by social consensus *)
  TARGET_HEIGHT,
    (* an index of the block header that the light client tries to verify *)
  IS_PRIMARY_CORRECT
    (* is primary correct? *)  

VARIABLES       (* see TypeOK below for the variable types *)
  state,        (* the current state of the light client *)
  nextHeight    (* the next height to explore by the light client *)
  
(* the light store *)
VARIABLES  
  fetchedLightBlocks, (* a function from heights to LightBlocks *)
  lightBlockStatus,   (* a function from heights to block statuses *)
  latestVerified      (* the latest verified block *)

(* the variables of the lite client *)  
lcvars == <<state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified>>  

(******************* Blockchain instance ***********************************)

\* the parameters that are propagated into Blockchain
CONSTANTS
  AllNodes
    (* a set of all nodes that can act as validators (correct and faulty) *)

\* the state variables of Blockchain, see Blockchain.tla for the details
VARIABLES tooManyFaults, chainHeight, minTrustedHeight, blockchain, Faulty

\* All the variables of Blockchain. For some reason, BC!vars does not work
bcvars == <<tooManyFaults, chainHeight, minTrustedHeight, blockchain, Faulty>>

(* Create an instance of Blockchain.
   We could write EXTENDS Blockchain, but then all the constants and state variables
   would be hidden inside the Blockchain module.
 *) 
ULTIMATE_HEIGHT == TARGET_HEIGHT + 1 
 
BC == INSTANCE Blockchain_A_1 WITH
  tooManyFaults <- tooManyFaults, height <- chainHeight,
  minTrustedHeight <- minTrustedHeight, blockchain <- blockchain, Faulty <- Faulty

(************************** Lite client ************************************)

(* the heights on which the light client is working *)  
HEIGHTS == TRUSTED_HEIGHT..TARGET_HEIGHT

(* the control states of the lite client *) 
States == { "working", "finishedSuccess", "finishedFailure" }

(**
 Check the precondition of ValidAndVerified.
 
 TODO: add a traceability tag
 *)
ValidAndVerifiedPre(trusted, untrusted) ==
  LET thdr == trusted.header
      uhdr == untrusted.header
  IN
  /\ BC!InTrustingPeriod(thdr)
  /\ thdr.height < uhdr.height
  /\ thdr.height + 1 = uhdr.height =>
     /\ thdr.NextVS = uhdr.VS
     /\ untrusted.Commits \subseteq uhdr.VS
     /\ LET TP == Cardinality(uhdr.VS)
            SP == Cardinality(untrusted.Commits)
        IN
        3 * SP > 2 * TP     
  (* TODO:
  trusted.Commit is a commit is for the header trusted.Header, i.e. it contains the correct hash of the header
  *)
  (* we cannot check these:
  untrusted.Header.Time < now + clockDrift
  the Time of trusted are smaller than the Time of untrusted,
  *)
  (* we do not have to check these:
  untrusted.Validators = hash(untrusted.Header.Validators)
  untrusted.NextValidators = hash(untrusted.Header.NextValidators)
   *)

(** Check that the commits in an untrusted block form 1/3 of the next validators
    in a trusted header
 *)
OneThird(trusted, untrusted) ==
  LET TP == Cardinality(trusted.header.NextVS)
      SP == Cardinality(untrusted.Commits \intersect trusted.header.NextVS)
  IN
  3 * SP > TP     

(**
 Check, whether an untrusted block is valid and verifiable w.r.t. a trusted header.
 
 TODO: add traceability
 *)   
ValidAndVerified(trusted, untrusted) ==
    IF ~ValidAndVerifiedPre(trusted, untrusted)
    THEN "FAILED_VERIFICATION"
    ELSE IF ~BC!InTrustingPeriod(untrusted.header)
    THEN "FAILED_TRUSTING_PERIOD" 
    ELSE IF untrusted.header.height = trusted.header.height + 1
             \/ OneThird(trusted, untrusted)
         THEN "OK"
         ELSE "CANNOT_VERIFY"

(*
 Initial states of the light client. No requests on the stack, no headers.
 *)
LCInit ==
    /\ state = "working"
    /\ nextHeight = TARGET_HEIGHT
    /\ LET trustedBlock == blockchain[TRUSTED_HEIGHT]
           trustedLightBlock == [header |-> trustedBlock, Commits |-> AllNodes]
       IN
        /\ fetchedLightBlocks = [h \in {TRUSTED_HEIGHT} |-> trustedLightBlock]
        /\ lightBlockStatus = [h \in {TRUSTED_HEIGHT} |-> "StateVerified"]
        /\ latestVerified = trustedLightBlock
        
LightStoreGetInto(block, height) ==
    block = fetchedLightBlocks[height]

IsCanonicalLightBlock(block, height) ==
    LET ref == blockchain[height]
        lastCommit ==
          IF height < ULTIMATE_HEIGHT
          THEN blockchain[height + 1].lastCommit
          ELSE blockchain[height].VS \* for the ultimate block 
    IN
    block = [header |-> ref, Commits |-> lastCommit]      

FetchLightBlockInto(block, height) ==
    IF IS_PRIMARY_CORRECT
    THEN IsCanonicalLightBlock(block, height)
    ELSE BC!IsProducableByFaulty(height, block)
    
LightStoreUpdateBlocks(lightBlocks, block) ==
    LET ht == block.header.height IN    
    [h \in DOMAIN lightBlocks \union {ht} |->
        IF h = ht THEN block ELSE lightBlocks[h]]
      
LightStoreUpdateStates(statuses, ht, blockState) ==
    [h \in DOMAIN statuses \union {ht} |->
        IF h = ht THEN blockState ELSE statuses[h]]      

ScheduleTo(newHeight, pNextHeight, pTargetHeight, pLatestVerified) ==
    LET ht == pLatestVerified.header.height IN
    \/ /\ ht = pNextHeight
       /\ ht < pTargetHeight
       /\ pNextHeight < newHeight
       /\ newHeight <= pTargetHeight
    \/ /\ ht < pNextHeight
       /\ ht < pTargetHeight
       /\ ht < newHeight
       /\ newHeight < pNextHeight
    \/ /\ ht = pTargetHeight
       /\ newHeight = pTargetHeight
    
LightLoop ==
    /\ latestVerified.header.height < TARGET_HEIGHT
    /\ \E current \in BC!LightBlocks:
        /\ IF nextHeight \in DOMAIN fetchedLightBlocks
           THEN /\ LightStoreGetInto(current, nextHeight)
                /\ UNCHANGED fetchedLightBlocks
           ELSE /\ FetchLightBlockInto(current, nextHeight)
                /\ fetchedLightBlocks' = LightStoreUpdateBlocks(fetchedLightBlocks, current)
        /\ LET verdict == ValidAndVerified(latestVerified, current) IN
           \/ /\ verdict = "OK"
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateVerified")
              /\ latestVerified' = current
              /\ state' =
                    IF latestVerified'.header.height < TARGET_HEIGHT
                    THEN "working"
                    ELSE "finishedSuccess"
              /\ \E newHeight \in HEIGHTS:
                 /\ ScheduleTo(newHeight, nextHeight, TARGET_HEIGHT, current)
                 /\ nextHeight' = newHeight 
           \/ /\ verdict = "CANNOT_VERIFY"
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateUnverified")
              /\ \E newHeight \in HEIGHTS:
                 /\ ScheduleTo(newHeight, nextHeight, TARGET_HEIGHT, latestVerified)
                 /\ nextHeight' = newHeight 
              /\ UNCHANGED <<latestVerified, state>>
           \/ /\ verdict \notin { "OK", "CANNOT_VERIFY" }
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateFailed")
              /\ state' = "finishedFailure"
              /\ UNCHANGED <<latestVerified, nextHeight>>

LightFinish ==
    /\ latestVerified.header.height >= TARGET_HEIGHT
    /\ state' = "finishedSuccess"
    /\ UNCHANGED <<nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified>>  
    
(*
 Actions of the light client: start or do bisection when receiving response.
 *)
LCNext ==
  \*\/ state /= "working" /\ UNCHANGED lcvars
  \/ /\ state = "working"
     /\ LightLoop \/ LightFinish 
            
            
(********************* Lite client + Environment + Blockchain *******************)
Init ==
    BC!Init /\ LCInit

(*
  A system step is made by one of two components: light client and  blockchain.
 *)
Next ==
    (* initialize the reference chain, no faults and time issues at this point *)
    \/ chainHeight < ULTIMATE_HEIGHT /\ BC!AdvanceChain /\ UNCHANGED lcvars
    (* advance time but still trust the ultimate block *)
    \/ chainHeight = ULTIMATE_HEIGHT /\ BC!AdvanceTime /\ ~tooManyFaults' /\ UNCHANGED lcvars
    (* introduce more faults but not too many *)
    \/ chainHeight = ULTIMATE_HEIGHT /\ BC!OneMoreFault /\ ~tooManyFaults' /\ UNCHANGED lcvars
    (* one more step by the light client *)
    \/ chainHeight = ULTIMATE_HEIGHT /\ state /= "finished" /\ LCNext /\ UNCHANGED bcvars

(************************* Types ******************************************)

TypeOK ==
    /\ state \in States
    /\ nextHeight \in HEIGHTS
    /\ latestVerified \in BC!LightBlocks
    /\ \E HS \in SUBSET HEIGHTS:
        /\ fetchedLightBlocks \in [HS -> BC!LightBlocks]
        /\ lightBlockStatus \in [HS -> {"StateVerified", "StateUnverified"}]

(************************* Properties ******************************************)

(* The properties to check *)
NeverFinish ==
    state = "working"

\* check this property to get an example of a terminating light client
NeverFinishNegative ==
    state /= "finishedFailure"
    
NeverFinishNegativeWhenTrusted ==
    (minTrustedHeight <= TRUSTED_HEIGHT) => state /= "finishedFailure"     

NeverFinishPositive ==
    state /= "finishedSuccess"

\* Correctness states that all the obtained headers are exactly like in the blockchain.
\* This formula is equivalent to Accuracy in the English spec, that is, A => B iff ~B => ~A.
\*
\* Lite Client Accuracy: If header h was not generated by an instance of Tendermint consensus,
\* then the lite client should never set trust(h) to true.
CorrectnessInv ==
    state = "finishedSuccess"
        => \A h \in DOMAIN fetchedLightBlocks:
             lightBlockStatus[h] = "StateVerified" =>
                fetchedLightBlocks[h].header = blockchain[h]

(*
\* CorrectnessInv holds only under the assumption of the Tendermint security model.
\* Hence, Correctness is restricted to the case when there are less than 1/3 of faulty validators.
Correctness ==
    []~tooManyFaults => CorrectnessInv

\* There are no two headers of the same height
NoDupsInv ==
    \A shdr1, shdr2 \in storedLightBlocks:
      (shdr1.header.height = shdr2.header.height) => (shdr1 = shdr2)

\* Check that the sequence of the headers in storedLightBlocks satisfies checkSupport pairwise
\* This property is easily violated, whenever a header cannot be trusted anymore.
StoredHeadersAreSound ==
    outEvent.type = "finished" /\ outEvent.verdict = TRUE
        =>
        \A left, right \in storedLightBlocks: \* for every pair of different stored headers
            \/ left.header.height >= right.header.height
               \* either there is a header between them
            \/ \E middle \in storedLightBlocks:
                /\ left.header.height < middle.header.height
                /\ middle.header.height < right.header.height
               \* or we can verify the right one using the left one
            \/ CheckSupport(left.header.height, right.header.height, left.header, right)

\* An improved version of StoredHeadersAreSound, assuming that a header may be not trusted
StoredHeadersAreSoundOrNotTrusted ==
    outEvent.type = "finished" /\ outEvent.verdict = TRUE
        =>
        \A left, right \in storedLightBlocks: \* for every pair of different stored headers
            \/ left.header.height >= right.header.height
               \* either there is a header between them
            \/ \E middle \in storedLightBlocks:
                /\ left.header.height < middle.header.height
                /\ middle.header.height < right.header.height
               \* or we can verify the right one using the left one
            \/ CheckSupport(left.header.height, right.header.height, left.header, right)
               \* or the left header is outside the trusting period, so no guarantees 
            \/ minTrustedHeight > left.header.height

\* The lite client must always terminate under the given pre-conditions.
\* E.g., assuming that the full node always replies.
TerminationPre ==
       \* the user and the full node take steps, if they can
    /\ WF_envvars(EnvNext)
       \* and the lite client takes steps, if it can
    /\ WF_lcvars(LCNext)
       \* eventually, the blockchain is sufficiently high, and
       \* the blockchain is never dead, see NeverStuck in Blockchain.tla
    /\ <>[](height >= minTrustedHeight /\ height >= TO_VERIFY_HEIGHT)

\* Given the preconditions, the lite client eventually terminates.        
Termination ==
  TerminationPre => <>(outEvent.type = "finished")        

\* This property states that whenever the light client finishes with a positive outcome,
\* the trusted header is still within the trusting period.
\* The current spec most likely violates this property.
PositiveBeforeTrustedHeaderExpires ==
    (outEvent.type = "finished" /\ outEvent.verdict = TRUE)
        => (minTrustedHeight <= TRUSTED_HEIGHT)

\* Lite Client Completeness: If header h was correctly generated by an instance
\* of Tendermint consensus (and its age is less than the trusting period),
\* then the lite client should eventually set trust(h) to true.
\*
\* Note that Completeness assumes that the lite client communicates with a correct full node.
\*
\* We decompose completeness into Termination (liveness) and Precision (safety).
\* Once again, Precision is an inverse version of the safety property in Completeness,
\* as A => B is logically equivalent to ~B => ~A. 
PrecisionInv ==
    (outEvent.type = "finished" /\ outEvent.verdict = FALSE)
      => \/ minTrustedHeight > TRUSTED_HEIGHT \* outside of the trusting period
         \/ \E shdr \in storedLightBlocks:
                 \* the full node lied to the lite client about the block header
              \/ shdr.header /= blockchain[shdr.header.height]
                 \* the full node lied to the lite client about the commits
              \/ shdr.Commits /= BC!VS(shdr.header)

\* the old invariant that was found to be buggy by TLC
PrecisionBuggyInv ==
    (outEvent.type = "finished" /\ outEvent.verdict = FALSE)
      => \/ minTrustedHeight > TRUSTED_HEIGHT \* outside of the trusting period
         \/ \E shdr \in storedLightBlocks:
              \* the full node lied to the lite client about the block header
              shdr.header /= blockchain[shdr.header.height]

\* Precision states that PrecisionInv always holds true.
\* When we independently check Termination and PrecisionInv,
\* there is no need to check Completeness.         
Completeness ==
    Termination /\ []PrecisionInv
*)    
=============================================================================
\* Modification History
\* Last modified Thu Jun 04 10:36:44 CEST 2020 by igor
\* Created Wed Oct 02 16:39:42 CEST 2019 by igor
