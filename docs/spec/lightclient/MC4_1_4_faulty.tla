---------------------------- MODULE MC4_1_4_faulty ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 4
TRUSTING_PERIOD == 14 \* two weeks 
IIS_PRIMARY_CORRECT == FALSE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  (*tooManyFaults,*) chainHeight, (*minTrustedHeight,*) now, blockchain, Faulty

INSTANCE Lightclient_A_1
==============================================================================
