------------------------- MODULE LightTests ---------------------------

EXTENDS Lightclient_002_draft

(* The light client history, which is the function mapping states 0..nprobes to the record with fields:
   - verified: the latest verified block in the previous state
   - current: the block that is being checked in the previous state
   - now: the time point in the previous state
   - verdict: the light client verdict in the previous state
*)
VARIABLE
  history

(* APALACHE annotations *)
a <: b == a \* type annotation

\* This predicate extends the LightClient Init predicate with history tracking
InitTest ==
  /\ Init
  /\ history = [ n \in {0} <: {Int} |->
     [ verified |-> prevVerified, current |-> prevCurrent, now |-> prevNow, verdict |-> prevVerdict ]]

\* This predicate extends the LightClient Next predicate with history tracking
NextTest ==
  /\ Next
  /\ history' = [ n \in DOMAIN history \union {nprobes'} |->
       IF n = nprobes' THEN
         [ verified |-> prevVerified', current |-> prevCurrent', now |-> prevNow', verdict |-> prevVerdict' ]
       ELSE history[n]
     ]

\* Some useful operators for writing tests

Valset(st) == history[st].current.header.VS


\* Test an execution that finishes with failure
TestFailure ==
    /\ state = "finishedFailure"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

\* Test an execution that finishes with success
TestSuccess ==
    /\ state = "finishedSuccess"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

\* This test never produces a counterexample; so the model should be corrected
TestFailedTrustingPeriod ==
   \E s \in DOMAIN history :
      history[s].verdict = "FAILED_TRUSTING_PERIOD"

TwoNotEnoughTrust ==
   \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

ThreeNotEnoughTrust ==
  \E s1, s2, s3 \in DOMAIN history :
       /\ s1 /= s2 /\ s2 /= s3 /\ s1 /= s3
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s3].verdict = "NOT_ENOUGH_TRUST"

\* Test an execution that finishes with success, and processes two headers with insufficient trust on the way
Test2NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ TwoNotEnoughTrust

\* Test an execution that finishes with failure, and processes two headers with insufficient trust on the way
Test2NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ TwoNotEnoughTrust


\* Test an execution that finishes with success, and processes three headers with insufficient trust on the way
Test3NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ ThreeNotEnoughTrust

\* Test an execution that finishes with failure, and processes three headers with insufficient trust on the way
Test3NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ ThreeNotEnoughTrust

\* Time-related tests

\* Test an execution where a header is received from the future
TestHeaderFromFuture ==
    /\ \E s \in DOMAIN history :
       history[s].now < history[s].current.header.time

\* Test an execution where the untrusted header time is before the trusted header time
TestUntrustedBeforeTrusted ==
    /\ \E s \in DOMAIN history :
       history[s].current.header.time < history[s].verified.header.time


\* Validator set tests

\* Test an execution where the validator sets differ at each step
TestValsetDifferentAllSteps ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \A s1, s2 \in DOMAIN history :
       s1 /= s2  =>
       history[s1].current.header.VS /= history[s2].current.header.VS

TestHalfValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(Valset(s1)) >= 3
        /\ 2 * Cardinality(Valset(s1) \intersect Valset(s2)) < Cardinality(Valset(s1))

TestHalfValsetChangesVerdictSuccess ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ history[s2].verdict = "SUCCESS"
        /\ Cardinality(Valset(s1)) >= 3
        /\ 2 * Cardinality(Valset(s1) \intersect Valset(s2)) < Cardinality(Valset(s1))

TestHalfValsetChangesVerdictNotEnoughTrust ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
        /\ Cardinality(Valset(s1)) >= 3
        /\ 2 * Cardinality(Valset(s1) \intersect Valset(s2)) < Cardinality(Valset(s1))

============================================================================
