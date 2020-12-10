------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n1"},
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 5
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 5
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n1"},
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 5,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 5
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> {},
      time |-> 4]]
/\ prevNow = 5
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 2 to State4 *)

State4 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n1"},
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n2"},
          height |-> 2,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 2]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 5,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 2]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1403
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]]
/\ prevNow = 5
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State5 *)

State5 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n1"},
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n2"},
          height |-> 2,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 2]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> { "n1", "n3", "n4" },
          VS |-> {"n1"},
          height |-> 3,
          lastCommit |-> {"n1"},
          time |-> 3]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 5,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 2]],
      now |-> 5,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> { "n1", "n3", "n4" },
              VS |-> {"n1"},
              height |-> 3,
              lastCommit |-> {"n1"},
              time |-> 3]],
      now |-> 1403,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 2]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 4 :> "StateFailed"
/\ nextHeight = 4
/\ now = 1403
/\ nprobes = 3
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> { "n1", "n3", "n4" },
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> {"n1"},
      time |-> 3]]
/\ prevNow = 1403
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 2]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 4
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["now"] > history[s$2]["verified"]["header"]["time"] + 1400
        /\ history[s$2]["current"]["header"]["time"] < history[s$2]["now"]))

================================================================================
\* Created by Apalache on Wed Nov 18 12:39:38 UTC 2020
\* https://github.com/informalsystems/apalache
