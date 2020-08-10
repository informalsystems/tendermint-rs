------------------------- MODULE LiteTests ---------------------------

EXTENDS Lightclient_A_1

TestFailure ==
    /\ state = "finishedFailure"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

TestFailureInv == ~TestFailure


TestSuccess ==
    /\ state = "finishedSuccess"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

TestSuccessInv == ~TestSuccess

============================================================================
