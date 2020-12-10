------------------------- MODULE MC50_2_faulty ---------------------------

AllNodes == {
  "n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9", "n10",
  "n11", "n12", "n13", "n14", "n15", "n16", "n17", "n18", "n19", "n20",
  "n21", "n22", "n23", "n24", "n25", "n26", "n27", "n28", "n29", "n30",
  "n31", "n32", "n33", "n34", "n35", "n36", "n37", "n38", "n39", "n40",
  "n41", "n42", "n43", "n44", "n45", "n46", "n47", "n48", "n49", "n50"
}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 2
TRUSTING_PERIOD == 1400 \* two weeks, one day is 100 time units :-)
IS_PRIMARY_CORRECT == FALSE

VARIABLES
  state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified,
  nprobes, now, blockchain, Faulty,
  prevVerified, prevCurrent, prevNow, prevVerdict,
  history

INSTANCE LightTests

============================================================================
