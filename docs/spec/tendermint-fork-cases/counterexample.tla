------------------------- MODULE counterexample -------------------------

EXTENDS MC_n4_f2

(* Initial state *)

State1 ==
/\ Proposer = 0 :> "f4" @@ 1 :> "c1" @@ 2 :> "c2"

(* Transition 0 to State2 *)

State2 ==
/\ Proposer = 0 :> "f4" @@ 1 :> "c1" @@ 2 :> "c2"
/\ decision = "c1" :> "None" @@ "c2" :> "None"
/\ evidence = {}
/\ lockedRound = "c1" :> -1 @@ "c2" :> -1
/\ lockedValue = "c1" :> "None" @@ "c2" :> "None"
/\ msgsPrecommit = 0 :> {}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"] }
  @@ 2 :> {}
/\ msgsPrevote = 0 :> {}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PREVOTE"] }
  @@ 2 :> {[id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"]}
/\ msgsPropose = 0
    :> {[proposal |-> "v1",
      round |-> 0,
      src |-> "f4",
      type |-> "PROPOSAL",
      validRound |-> -1]}
  @@ 1 :> {}
  @@ 2
    :> { [proposal |-> "v0",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> 2],
      [proposal |-> "v2",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1] }
/\ round = "c1" :> 0 @@ "c2" :> 0
/\ step = "c1" :> "PROPOSE" @@ "c2" :> "PROPOSE"
/\ validRound = "c1" :> -1 @@ "c2" :> -1
/\ validValue = "c1" :> "None" @@ "c2" :> "None"

(* Transition 1 to State3 *)

State3 ==
/\ Proposer = 0 :> "f4" @@ 1 :> "c1" @@ 2 :> "c2"
/\ decision = "c1" :> "None" @@ "c2" :> "None"
/\ evidence = {[proposal |-> "v1",
  round |-> 0,
  src |-> "f4",
  type |-> "PROPOSAL",
  validRound |-> -1]}
/\ lockedRound = "c1" :> -1 @@ "c2" :> -1
/\ lockedValue = "c1" :> "None" @@ "c2" :> "None"
/\ msgsPrecommit = 0 :> {}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"] }
  @@ 2 :> {}
/\ msgsPrevote = 0 :> {[id |-> "v1", round |-> 0, src |-> "c2", type |-> "PREVOTE"]}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PREVOTE"] }
  @@ 2 :> {[id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"]}
/\ msgsPropose = 0
    :> {[proposal |-> "v1",
      round |-> 0,
      src |-> "f4",
      type |-> "PROPOSAL",
      validRound |-> -1]}
  @@ 1 :> {}
  @@ 2
    :> { [proposal |-> "v0",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> 2],
      [proposal |-> "v2",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1] }
/\ round = "c1" :> 0 @@ "c2" :> 0
/\ step = "c1" :> "PROPOSE" @@ "c2" :> "PREVOTE"
/\ validRound = "c1" :> -1 @@ "c2" :> -1
/\ validValue = "c1" :> "None" @@ "c2" :> "None"

(* Transition 5 to State4 *)

State4 ==
/\ Proposer = 0 :> "f4" @@ 1 :> "c1" @@ 2 :> "c2"
/\ decision = "c1" :> "None" @@ "c2" :> "None"
/\ evidence = { [id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"],
  [proposal |-> "v0",
    round |-> 2,
    src |-> "f3",
    type |-> "PROPOSAL",
    validRound |-> 1],
  [proposal |-> "v1",
    round |-> 0,
    src |-> "f4",
    type |-> "PROPOSAL",
    validRound |-> -1] }
/\ lockedRound = "c1" :> -1 @@ "c2" :> -1
/\ lockedValue = "c1" :> "None" @@ "c2" :> "None"
/\ msgsPrecommit = 0 :> {}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"] }
  @@ 2 :> {}
/\ msgsPrevote = 0 :> {[id |-> "v1", round |-> 0, src |-> "c2", type |-> "PREVOTE"]}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PREVOTE"] }
  @@ 2 :> {[id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"]}
/\ msgsPropose = 0
    :> {[proposal |-> "v1",
      round |-> 0,
      src |-> "f4",
      type |-> "PROPOSAL",
      validRound |-> -1]}
  @@ 1 :> {}
  @@ 2
    :> { [proposal |-> "v0",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> 2],
      [proposal |-> "v2",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1] }
/\ round = "c1" :> 0 @@ "c2" :> 2
/\ step = "c1" :> "PROPOSE" @@ "c2" :> "PROPOSE"
/\ validRound = "c1" :> -1 @@ "c2" :> -1
/\ validValue = "c1" :> "None" @@ "c2" :> "None"

(* Transition 8 to State5 *)

State5 ==
/\ Proposer = 0 :> "f4" @@ 1 :> "c1" @@ 2 :> "c2"
/\ decision = "c1" :> "None" @@ "c2" :> "None"
/\ evidence = { [id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"],
  [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
  [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
  [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
  [proposal |-> "v0",
    round |-> 2,
    src |-> "f3",
    type |-> "PROPOSAL",
    validRound |-> 1],
  [proposal |-> "v1",
    round |-> 0,
    src |-> "f4",
    type |-> "PROPOSAL",
    validRound |-> -1] }
/\ lockedRound = "c1" :> -1 @@ "c2" :> -1
/\ lockedValue = "c1" :> "None" @@ "c2" :> "None"
/\ msgsPrecommit = 0 :> {}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PRECOMMIT"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PRECOMMIT"] }
  @@ 2 :> {}
/\ msgsPrevote = 0 :> {[id |-> "v1", round |-> 0, src |-> "c2", type |-> "PREVOTE"]}
  @@ 1
    :> { [id |-> "v0", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f3", type |-> "PREVOTE"],
      [id |-> "v1", round |-> 1, src |-> "f4", type |-> "PREVOTE"],
      [id |-> "v2", round |-> 1, src |-> "f3", type |-> "PREVOTE"] }
  @@ 2 :> {[id |-> "v0", round |-> 2, src |-> "f4", type |-> "PREVOTE"]}
/\ msgsPropose = 0
    :> {[proposal |-> "v1",
      round |-> 0,
      src |-> "f4",
      type |-> "PROPOSAL",
      validRound |-> -1]}
  @@ 1 :> {}
  @@ 2
    :> { [proposal |-> "v0",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f3",
        type |-> "PROPOSAL",
        validRound |-> 1],
      [proposal |-> "v1",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> 2],
      [proposal |-> "v2",
        round |-> 2,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1] }
/\ round = "c1" :> 1 @@ "c2" :> 2
/\ step = "c1" :> "PROPOSE" @@ "c2" :> "PROPOSE"
/\ validRound = "c1" :> -1 @@ "c2" :> -1
/\ validValue = "c1" :> "None" @@ "c2" :> "None"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  BMC!Skolem((\E p$24 \in { "f3", "f4" }:
    BMC!Skolem((\E r1$2 \in 0 .. 2:
      BMC!Skolem((\E r2$2 \in 0 .. 2:
        r1$2 < r2$2
          /\ BMC!Skolem((\E v1$2 \in { "v0", "v1" }:
            BMC!Skolem((\E v2$2 \in { "v0", "v1" }:
              ~(v1$2 = v2$2)
                /\ [type |-> "PRECOMMIT",
                  src |-> p$24,
                  round |-> r1$2,
                  id |-> v1$2]
                  <: [type |-> STRING,
                    src |-> STRING,
                    round |-> Int,
                    proposal |-> STRING,
                    validRound |-> Int,
                    id |-> STRING]
                  \in evidence
                /\ [type |-> "PREVOTE",
                  src |-> p$24,
                  round |-> r2$2,
                  id |-> v2$2]
                  <: [type |-> STRING,
                    src |-> STRING,
                    round |-> Int,
                    proposal |-> STRING,
                    validRound |-> Int,
                    id |-> STRING]
                  \in evidence
                /\ (\A r$8 \in {
                  rnd$2 \in 0 .. 2:
                    r1$2 <= rnd$2 /\ rnd$2 < r2$2
                }:
                  LET prevotes$2 ==
                    {
                      m$17 \in evidence:
                        (m$17["type"] = "PREVOTE" /\ m$17["round"] = r$8)
                          /\ m$17["id"] = v2$2
                    }
                  IN
                  Cardinality((prevotes$2)) < 3)))))))))))

================================================================================
\* Created by Apalache on Fri Dec 04 23:48:58 CET 2020
\* https://github.com/informalsystems/apalache
