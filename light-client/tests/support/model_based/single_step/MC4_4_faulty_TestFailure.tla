------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3" },
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n4"},
      time |-> 4]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3" },
      height |-> 4,
      lastCommit |-> {"n4"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1396
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1396
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3" },
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n4"},
      time |-> 4]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3" },
      height |-> 4,
      lastCommit |-> {"n4"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> { "n1", "n2", "n3" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3" },
          height |-> 4,
          lastCommit |-> {"n4"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2", "n3" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3" },
              height |-> 4,
              lastCommit |-> {"n4"},
              time |-> 5]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1396
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3" },
      height |-> 4,
      lastCommit |-> {"n4"},
      time |-> 5]]
/\ prevNow = 1396
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State4 *)

State4 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3" },
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n4"},
      time |-> 4]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3" },
      height |-> 4,
      lastCommit |-> {"n4"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n4"},
          VS |-> {"n4"},
          height |-> 2,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 3]]
  @@ 4
    :> [Commits |-> { "n1", "n2", "n3" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3" },
          height |-> 4,
          lastCommit |-> {"n4"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2", "n3" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3" },
              height |-> 4,
              lastCommit |-> {"n4"},
              time |-> 5]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n4"},
              VS |-> {"n4"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 3]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1404
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]]
/\ prevNow = 1396
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State5 *)

State5 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3" },
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n4"},
      time |-> 4]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> { "n1", "n2", "n3" },
      height |-> 4,
      lastCommit |-> {"n4"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n4"},
          VS |-> {"n4"},
          height |-> 2,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 3]]
  @@ 3
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> { "n1", "n2", "n3" },
          VS |-> {"n4"},
          height |-> 3,
          lastCommit |-> {"n4"},
          time |-> 4]]
  @@ 4
    :> [Commits |-> { "n1", "n2", "n3" },
      header |->
        [NextVS |-> {"n4"},
          VS |-> { "n1", "n2", "n3" },
          height |-> 4,
          lastCommit |-> {"n4"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2", "n3" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3" },
              height |-> 4,
              lastCommit |-> {"n4"},
              time |-> 5]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n4"},
              VS |-> {"n4"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 3]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n4"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> { "n1", "n2", "n3" },
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n4"},
              time |-> 4]],
      now |-> 1404,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n4"},
              VS |-> {"n4"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 3]]]
/\ latestVerified = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateFailed"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1404
/\ nprobes = 3
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> { "n1", "n2", "n3" },
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n4"},
      time |-> 4]]
/\ prevNow = 1404
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n4"},
      VS |-> {"n4"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 3]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedFailure" /\ Cardinality((DOMAIN fetchedLightBlocks)) = 4

================================================================================
\* Created by Apalache on Wed Oct 21 08:44:51 UTC 2020
\* https://github.com/informalsystems/apalache
