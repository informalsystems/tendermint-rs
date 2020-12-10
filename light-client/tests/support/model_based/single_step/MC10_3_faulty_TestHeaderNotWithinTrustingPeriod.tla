------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n2", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n5" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4", "n5", "n7", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n7" },
      VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n5" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n7" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1396
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1396
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n2", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n5" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4", "n5", "n7", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n7" },
      VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n5" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n7" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n10", "n6", "n7", "n8" },
      header |->
        [NextVS |-> { "n3", "n4", "n6", "n7" },
          VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
          height |-> 3,
          lastCommit |-> { "n3", "n5" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n10", "n6", "n7", "n8" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n7" },
              VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n3", "n5" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1402
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n10", "n6", "n7", "n8" },
  header |->
    [NextVS |-> { "n3", "n4", "n6", "n7" },
      VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n5" },
      time |-> 4]]
/\ prevNow = 1396
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State4 *)

State4 ==
/\ Faulty = { "n2", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n5" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3", "n4", "n5", "n7", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n7" },
      VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n5" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n7" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n10", "n6", "n7", "n8" },
      header |->
        [NextVS |-> { "n3", "n4", "n6", "n7" },
          VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
          height |-> 3,
          lastCommit |-> { "n3", "n5" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {},
      header |->
        [NextVS |-> { "n1", "n10", "n2", "n3", "n7", "n8", "n9" },
          VS |-> { "n10", "n3", "n5", "n9" },
          height |-> 4,
          lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8" },
          time |-> 2]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n10", "n6", "n7", "n8" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n7" },
              VS |-> { "n1", "n10", "n2", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n3", "n5" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {},
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n7", "n8", "n9" },
              VS |-> { "n10", "n3", "n5", "n9" },
              height |-> 4,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8" },
              time |-> 2]],
      now |-> 1402,
      verdict |-> "INVALID",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateFailed" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1402
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {},
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n7", "n8", "n9" },
      VS |-> { "n10", "n3", "n5", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]]
/\ prevNow = 1402
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 3
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["now"] > history[s$2]["verified"]["header"]["time"] + 1400
        /\ history[s$2]["current"]["header"]["time"] < history[s$2]["now"]))

================================================================================
\* Created by Apalache on Wed Nov 18 13:56:19 UTC 2020
\* https://github.com/informalsystems/apalache
