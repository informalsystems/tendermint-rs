------------------------- MODULE MC10_3_faulty ---------------------------

AllNodes == {"n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9", "n10"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 3
TRUSTING_PERIOD == 1400 \* two weeks, one day is 100 time units :-)
IS_PRIMARY_CORRECT == FALSE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  nprobes, now, blockchain, Faulty,
  prevVerified, prevCurrent, prevNow, prevVerdict,
  history

INSTANCE LightTests

============================================================================
