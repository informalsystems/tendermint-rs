------------------------- MODULE LightTests ---------------------------

EXTENDS Lightclient_A_1

(* The light client history, which is the function mapping states 1..nprobes to the record with fields:
   - verified: the latest verified block in the previous state
   - current: the block that is being checked in the previous state
   - now: the time point in the previous state
   - verdict: the light client verdict in the previous state
*)
VARIABLE
  history

historyState ==
  [ verified |-> prevVerified, current |-> prevCurrent, now |-> prevNow, verdict |-> prevVerdict ]

(* APALACHE annotations *)
a <: b == a \* type annotation

InitTest ==
  /\ Init
  /\ history = [ n \in {} <: {Int} |-> historyState ]

NextTest ==
  /\ Next
  /\ history' = [ n \in DOMAIN history \union {nprobes} |-> IF n = nprobes THEN historyState ELSE history[n]]

TestFailure ==
    /\ state = "finishedFailure"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

TestSuccess ==
    /\ state = "finishedSuccess"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

\* This test never produces a counterexample; so the model should be corrected
TestFailedTrustingPeriod ==
   \E s \in DOMAIN history :
      history[s].verdict = "FAILED_TRUSTING_PERIOD"

Test2NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

Test2NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

Test3NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2, s3 \in DOMAIN history :
       /\ s1 /= s2 /\ s2 /= s3 /\ s1 /= s3
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s3].verdict = "NOT_ENOUGH_TRUST"

============================================================================
