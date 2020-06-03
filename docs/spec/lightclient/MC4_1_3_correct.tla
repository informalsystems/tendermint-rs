------------------------- MODULE MC4_1_3_correct ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 3
IS_PRIMARY_CORRECT == TRUE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  tooManyFaults, chainHeight, minTrustedHeight, blockchain, Faulty

INSTANCE Lightclient_A_1
============================================================================
