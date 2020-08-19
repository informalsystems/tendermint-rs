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

Test2CannotVerifySuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "CANNOT_VERIFY"
       /\ history[s2].verdict = "CANNOT_VERIFY"

Test2CannotVerifySuccessInv == ~Test2CannotVerifySuccess

Test2CannotVerifyFailure ==
    /\ state = "finishedFailure"
    /\ \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "CANNOT_VERIFY"
       /\ history[s2].verdict = "CANNOT_VERIFY"

Test2CannotVerifyFailureInv == ~Test2CannotVerifyFailure

Test3CannotVerifySuccess ==
    /\ state = "finishedSuccess"
    /\ \E s1, s2, s3 \in DOMAIN history :
       /\ s1 /= s2 /\ s2 /= s3 /\ s1 /= s3
       /\ history[s1].verdict = "CANNOT_VERIFY"
       /\ history[s2].verdict = "CANNOT_VERIFY"
       /\ history[s3].verdict = "CANNOT_VERIFY"

Test3CannotVerifySuccessInv == ~Test3CannotVerifySuccess

============================================================================
