---------------------------- MODULE MC4_1_4_faulty ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 4
TRUSTING_PERIOD == 1400 \* two weeks, one week is 100 time units :-)
IS_PRIMARY_CORRECT == FALSE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  (*tooManyFaults,*) chainHeight, (*minTrustedHeight,*) now, blockchain, Faulty

INSTANCE Lightclient_A_1
==============================================================================
