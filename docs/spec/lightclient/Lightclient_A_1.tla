-------------------------- MODULE Lightclient_A_1 ----------------------------
(**
 * A state-machine specification of the lite client, following the English spec:
 *
 * https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md
 *) 

EXTENDS Integers, FiniteSets

\* the parameters of Light Client
CONSTANTS
  TRUSTED_HEIGHT,
    (* an index of the block header that the light client trusts by social consensus *)
  TARGET_HEIGHT,
    (* an index of the block header that the light client tries to verify *)
  TRUSTING_PERIOD,
    (* the period within which the validators are trusted *)
  IS_PRIMARY_CORRECT
    (* is primary correct? *)  

VARIABLES       (* see TypeOK below for the variable types *)
  state,        (* the current state of the light client *)
  nextHeight,   (* the next height to explore by the light client *)
  nprobes       (* the lite client iteration, or the number of block tests *)
  
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
VARIABLES (*tooManyFaults,*) chainHeight, (*minTrustedHeight,*) now, blockchain, Faulty

\* All the variables of Blockchain. For some reason, BC!vars does not work
bcvars == <<(*tooManyFaults,*) chainHeight, (*minTrustedHeight,*) now, blockchain, Faulty>>

(* Create an instance of Blockchain.
   We could write EXTENDS Blockchain, but then all the constants and state variables
   would be hidden inside the Blockchain module.
 *) 
ULTIMATE_HEIGHT == TARGET_HEIGHT + 1 
 
BC == INSTANCE Blockchain_A_1 WITH
  (*tooManyFaults <- tooManyFaults,*) height <- chainHeight,
  (*minTrustedHeight <- minTrustedHeight,*) now <- now, blockchain <- blockchain, Faulty <- Faulty

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
     \* the trusted block has been created earlier (no drift here)
     (* the English spec says:
        untrusted.Header.Time < now + clockDrift
        the Time of trusted are smaller than the Time of untrusted *)
  /\ thdr.time <= uhdr.time
  /\ thdr.height + 1 = uhdr.height =>
     /\ thdr.NextVS = uhdr.VS
     /\ untrusted.Commits \subseteq uhdr.VS
     /\ LET TP == Cardinality(uhdr.VS)
            SP == Cardinality(untrusted.Commits)
        IN
        3 * SP > 2 * TP     
  (* TODO:
  trusted.Commit is a commit is for the header trusted.Header,
  i.e. it contains the correct hash of the header
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
    /\ nprobes = 0
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
        /\ nprobes' = nprobes + 1 \* one more test
        /\ LET verdict == ValidAndVerified(latestVerified, current) IN
           CASE verdict = "OK" ->
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateVerified")
              /\ latestVerified' = current
              /\ state' =
                    IF latestVerified'.header.height < TARGET_HEIGHT
                    THEN "working"
                    ELSE "finishedSuccess"
              /\ \E newHeight \in HEIGHTS:
                 /\ ScheduleTo(newHeight, nextHeight, TARGET_HEIGHT, current)
                 /\ nextHeight' = newHeight
                  
           [] verdict = "CANNOT_VERIFY" ->
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateUnverified")
              /\ \E newHeight \in HEIGHTS:
                 /\ ScheduleTo(newHeight, nextHeight, TARGET_HEIGHT, latestVerified)
                 /\ nextHeight' = newHeight 
              /\ UNCHANGED <<latestVerified, state>>
              
           [] OTHER ->
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateFailed")
              /\ state' = "finishedFailure"
              /\ UNCHANGED <<latestVerified, nextHeight>>

LightFinish ==
    /\ latestVerified.header.height >= TARGET_HEIGHT
    /\ state' = "finishedSuccess"
    /\ UNCHANGED <<nextHeight, nprobes, fetchedLightBlocks, lightBlockStatus, latestVerified>>  
    
(*
 Actions of the light client: start or do bisection when receiving response.
 *)
LCNext ==
  \*\/ state /= "working" /\ UNCHANGED lcvars
  \/ /\ state = "working"
     /\ LightLoop \/ LightFinish 
            
            
(********************* Lite client + Blockchain *******************)
Init ==
    \* the blockchain is initialized immediately to the ULTIMATE_HEIGHT
    /\ BC!InitToHeight
    \* the light client starts
    /\ LCInit

(*
  The system step is very simple. The light client makes one iteration.
  Simultaneously, the global clock may advance.
 *)
Next ==
    /\ state /= "finished"
    /\ LCNext         \* the light client makes one step
    /\ BC!AdvanceTime \* the global clock is advanced by zero or more time units
    /\ UNCHANGED bcvars

(************************* Types ******************************************)

TypeOK ==
    /\ state \in States
    /\ nextHeight \in HEIGHTS
    /\ latestVerified \in BC!LightBlocks
    /\ \E HS \in SUBSET HEIGHTS:
        /\ fetchedLightBlocks \in [HS -> BC!LightBlocks]
        /\ lightBlockStatus
             \in [HS -> {"StateVerified", "StateUnverified", "StateFailed"}]

(************************* Properties ******************************************)

(* The properties to check *)
\* this invariant candidate is false    
NeverFinish ==
    state = "working"

\* this invariant candidate is false    
NeverFinishNegative ==
    state /= "finishedFailure"

\* This invariant holds true, when the primary is correct. 
\* This invariant candidate is false when the primary is faulty.    
NeverFinishNegativeWhenTrusted ==
    (*(minTrustedHeight <= TRUSTED_HEIGHT)*)
    BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
      => state /= "finishedFailure"     

\* this invariant candidate is false    
NeverFinishPositive ==
    state /= "finishedSuccess"

\* Correctness states that all the obtained headers are exactly like in the blockchain.
\* This formula is equivalent to Accuracy in the English spec, that is, A => B iff ~B => ~A.
\*
\* Lite Client Accuracy: If header h was not generated by an instance of Tendermint consensus,
\* then the lite client should never set trust(h) to true.
CorrectnessInv ==
    state = "finishedSuccess" =>
      \A h \in DOMAIN fetchedLightBlocks:
         lightBlockStatus[h] = "StateVerified" =>
           fetchedLightBlocks[h].header = blockchain[h]

\* Check that the sequence of the headers in storedLightBlocks satisfies ValidAndVerified = "OK" pairwise
\* This property is easily violated, whenever a header cannot be trusted anymore.
StoredHeadersAreSound ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks: \* for every pair of different stored headers
            \/ lh >= rh
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh
               \* or we can verify the right one using the left one
            \/ "OK" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])

\* An improved version of StoredHeadersAreSound, assuming that a header may be not trusted.
\* This invariant candidate is also violated,
\* as there may be some unverified blocks left in the middle.
StoredHeadersAreSoundOrNotTrusted ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks: \* for every pair of different stored headers
            \/ lh >= rh
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh
               \* or we can verify the right one using the left one
            \/ "OK" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])
               \* or the left header is outside the trusting period, so no guarantees
            \/ ~BC!InTrustingPeriod(fetchedLightBlocks[lh].header) 

\* An improved version of StoredHeadersAreSound, assuming that a header may be not trusted.
\* This invariant candidate is also violated,
\* as there may be some unverified blocks left in the middle.
VerifiedStoredHeadersAreSoundOrNotTrusted ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks:
                \* for every pair of different stored headers
            \/ lh >= rh
            \/ lightBlockStatus[lh] = "StateUnverified"
            \/ lightBlockStatus[rh] = "StateUnverified"
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh /\ lightBlockStatus[mh] = "StateVerified"
               \* or we can verify the right one using the left one
            \/ "OK" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])
               \* or the left header is outside the trusting period, so no guarantees
            \/ ~BC!InTrustingPeriod(fetchedLightBlocks[lh].header)

\* When the light client terminates, there is no failed blocks            
NoFailedBlocksOnSuccess ==
    state = "finishedSuccess" =>
        \A h \in DOMAIN fetchedLightBlocks:
            lightBlockStatus[h] /= "StateFailed"            

\* This property states that whenever the light client finishes with a positive outcome,
\* the trusted header is still within the trusting period.
\* We expect this property to be violated. And Apalache shows us a counterexample.
PositiveBeforeTrustedHeaderExpires ==
    (state = "finishedSuccess") => BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    
\* If the primary is correct and the initial trusted block has not expired,
\* then whenever the algorithm terminates, it reports "success"    
CorrectPrimaryAndTimeliness ==
  (BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    /\ state /= "working" /\ IS_PRIMARY_CORRECT) =>
      state = "finishedSuccess"     

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
    (state = "finishedFailure")
      => \/ ~BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT]) \* outside of the trusting period
         \/ \E h \in DOMAIN fetchedLightBlocks:
            LET lightBlock == fetchedLightBlocks[h] IN
                 \* the full node lied to the lite client about the block header
              \/ lightBlock.header /= blockchain[h]
                 \* the full node lied to the lite client about the commits
              \/ lightBlock.Commits /= lightBlock.header.VS

\* the old invariant that was found to be buggy by TLC
PrecisionBuggyInv ==
    (state = "finishedFailure")
      => \/ ~BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT]) \* outside of the trusting period
         \/ \E h \in DOMAIN fetchedLightBlocks:
            LET lightBlock == fetchedLightBlocks[h] IN
            \* the full node lied to the lite client about the block header
            lightBlock.header /= blockchain[h]

\* the worst complexity
Complexity ==
    LET N == TARGET_HEIGHT - TRUSTED_HEIGHT + 1 IN
    state /= "working" =>
        (2 * nprobes <= N * (N - 1)) 

(*
 We omit termination, as the algorithm deadlocks in the end.
 So termination can be demonstrated by finding a deadlock.
 Of course, one has to analyze the deadlocked state and see that
 the algorithm has indeed terminated there.
*)

(*

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

\* Precision states that PrecisionInv always holds true.
\* When we independently check Termination and PrecisionInv,
\* there is no need to check Completeness.         
Completeness ==
    Termination /\ []PrecisionInv
*)    
=============================================================================
\* Modification History
\* Last modified Fri Jun 05 16:54:49 CEST 2020 by igor
\* Created Wed Oct 02 16:39:42 CEST 2019 by igor
