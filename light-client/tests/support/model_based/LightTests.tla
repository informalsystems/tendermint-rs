------------------------- MODULE LightTests ---------------------------

EXTENDS Lightclient_A_1

TestFailure ==
    /\ state = "finishedFailure"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

TestFailureInv == ~TestFailure


TestSuccess ==
    /\ state = "finishedSuccess"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

TestSuccessInv == ~TestSuccess

\* This test never produces a counterexample; so the model should be corrected
TestFailedTrustingPeriod ==
   \E s \in DOMAIN history :
      history[s].verdict = "FAILED_TRUSTING_PERIOD"

TestFailedTrustingPeriodInv == ~TestFailedTrustingPeriod

Test2NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

Test2NotEnoughTrustSuccessInv == ~Test2NotEnoughTrustSuccess

Test2NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

Test2NotEnoughTrustFailureInv == ~Test2NotEnoughTrustFailure

Test3NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2, s3 \in DOMAIN history :
       /\ s1 /= s2 /\ s2 /= s3 /\ s1 /= s3
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s3].verdict = "NOT_ENOUGH_TRUST"

Test3NotEnoughTrustSuccessInv == ~Test3NotEnoughTrustSuccess

============================================================================
