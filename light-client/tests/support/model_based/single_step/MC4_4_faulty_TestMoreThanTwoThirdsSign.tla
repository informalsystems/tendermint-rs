------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {"n1"}
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 5,
      lastCommit |-> { "n2", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1400
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {"n1"}
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 5,
      lastCommit |-> { "n2", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 5
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n1"},
          height |-> 5,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n1"},
              height |-> 5,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 3]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1400
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n1"},
  header |->
    [NextVS |-> {"n1"},
      VS |-> {"n1"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]]
/\ prevNow = 1400
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 2 to State4 *)

State4 ==
/\ Faulty = {"n1"}
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 5,
      lastCommit |-> { "n2", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n2", "n4" },
      header |->
        [NextVS |-> { "n2", "n4" },
          VS |-> { "n2", "n4" },
          height |-> 3,
          lastCommit |-> { "n3", "n4" },
          time |-> 3]]
  @@ 5
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n1"},
          height |-> 5,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n1"},
              height |-> 5,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 3]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n4" },
              VS |-> { "n2", "n4" },
              height |-> 3,
              lastCommit |-> { "n3", "n4" },
              time |-> 3]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1400
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {"n1"}
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 5,
      lastCommit |-> { "n2", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n2", "n4" },
      header |->
        [NextVS |-> { "n2", "n4" },
          VS |-> { "n2", "n4" },
          height |-> 3,
          lastCommit |-> { "n3", "n4" },
          time |-> 3]]
  @@ 4
    :> [Commits |-> { "n2", "n4" },
      header |->
        [NextVS |-> { "n2", "n3" },
          VS |-> { "n2", "n4" },
          height |-> 4,
          lastCommit |-> { "n2", "n4" },
          time |-> 4]]
  @@ 5
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n1"},
          height |-> 5,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n1"},
              height |-> 5,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 3]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n4" },
              VS |-> { "n2", "n4" },
              height |-> 3,
              lastCommit |-> { "n3", "n4" },
              time |-> 3]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n2", "n4" },
              height |-> 4,
              lastCommit |-> { "n2", "n4" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n4" },
              VS |-> { "n2", "n4" },
              height |-> 3,
              lastCommit |-> { "n3", "n4" },
              time |-> 3]]]
/\ latestVerified = [Commits |-> { "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateVerified" @@ 4 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1400
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n2", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n4" },
      time |-> 4]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n4" },
      VS |-> { "n2", "n4" },
      height |-> 3,
      lastCommit |-> { "n3", "n4" },
      time |-> 3]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 4
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      3 * Cardinality(history[s$2]["current"]["Commits"])
        >= 2 * Cardinality(history[s$2]["current"]["header"]["VS"])))

================================================================================
\* Created by Apalache on Wed Nov 18 12:42:51 UTC 2020
\* https://github.com/informalsystems/apalache
