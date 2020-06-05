------------------------- MODULE MC4_1_5_correct ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 5
TRUSTING_PERIOD == 14 \* two weeks 
IS_PRIMARY_CORRECT == TRUE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  (*tooManyFaults,*) chainHeight, (*minTrustedHeight,*) now, blockchain, Faulty

INSTANCE Lightclient_A_1
============================================================================
