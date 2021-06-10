------------------------- MODULE MC4_4_faulty ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 4
TRUSTING_PERIOD == 1400 \* two weeks, one day is 100 time units :-)
IS_PRIMARY_CORRECT == FALSE

\* @typeAlias: BLOCKHEADER = [height: Int, time: Int, lastCommit: Set(Str), VS: Set(Str), NextVS: Set(Str)];
\* @typeAlias: BLOCK = [header: BLOCKHEADER, Commits: Set(Str)];
\* @typeAlias: BLOCKSTATUS = Int -> Str;
\* @typeAlias: BLOCKS = Int -> BLOCK;
MC4TypeAliases == TRUE

VARIABLES
  \* @type: Str;
  state,
  \* @type: Int;
  nextHeight,
  \* @type: Int -> BLOCK;
  fetchedLightBlocks,
  \* @type: BLOCKSTATUS;
  lightBlockStatus,
  \* @type: BLOCK;
  latestVerified,
  \* @type: Int;
  nprobes,
  \* @type: Int;
  now,
  \* @type: Int -> BLOCKHEADER;
  blockchain,
  \* @type: Set(Str);
  Faulty,
  \* @type: BLOCKHEADER;
  prevVerified,
  \* @type: BLOCK;
  prevCurrent,
  \* @type: Int;
  prevNow,
  \* @type: Str;
  prevVerdict,
  \* @type: Int -> [verified: BLOCK, current: BLOCK, now: Int, verdict: Str];
  history

INSTANCE LightTests

============================================================================
