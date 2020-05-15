------------------------------- MODULE MC_1_1_5 ------------------------------------

CORRECT == {"c1"}
FAULTY == {"f2"}
MAX_HEIGHT == 5
PEER_MAX_REQUESTS == 2
TARGET_PENDING == 3

VARIABLES
    state, blockPool, peersState, turn, inMsg, outMsg

INSTANCE fastsync_apalache    
========================================================================================
