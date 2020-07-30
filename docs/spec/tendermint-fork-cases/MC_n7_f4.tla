----------------------------- MODULE MC_n7_f4 -------------------------------
CONSTANT Proposer \* the proposer function from 0..NRounds to 1..N

\* the variables declared in TendermintAcc3
VARIABLES
  round, step, decision, lockedValue, lockedRound, validValue, validRound,
  msgsPropose, msgsPrevote, msgsPrecommit, evidence

\* an operator for type annotations
a <: b == a

INSTANCE TendermintAccDebug3 WITH
  Corr <- {"c1", "c2", "c3"},
  Defective <- {"f4", "f5"},
  Byzantine <- {"f6", "f7"},
  N <- 7,
  T <- 2,
  ValidValues <- { "v0", "v1" },
  InvalidValues <- {"v2"},
  MaxRound <- 2

\* run Apalache with --cinit=ConstInit
ConstInit == \* the proposer is arbitrary -- works for safety
  Proposer \in [Rounds -> AllProcs]

=============================================================================    
