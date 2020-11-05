------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n2", "n4" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n3", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n5", "n9" },
      VS |-> { "n1", "n10", "n3", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n4", "n7" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n2", "n3", "n5", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n3", "n7", "n8" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 4,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 4
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 4
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State3 *)

State3 ==
/\ Faulty = { "n2", "n4" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n3", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n5", "n9" },
      VS |-> { "n1", "n10", "n3", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n4", "n7" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n2", "n3", "n5", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n3", "n7", "n8" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |-> {},
      header |->
        [NextVS |-> { "n1", "n10", "n5", "n6" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7" },
          height |-> 1,
          lastCommit |-> { "n2", "n3", "n4", "n6" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 4,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {},
          header |->
            [NextVS |-> { "n1", "n10", "n5", "n6" },
              VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7" },
              height |-> 1,
              lastCommit |-> { "n2", "n3", "n4", "n6" },
              time |-> 5]],
      now |-> 4,
      verdict |-> "INVALID",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateFailed"
/\ nextHeight = 3
/\ now = 4
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {},
  header |->
    [NextVS |-> { "n1", "n10", "n5", "n6" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7" },
      height |-> 1,
      lastCommit |-> { "n2", "n3", "n4", "n6" },
      time |-> 5]]
/\ prevNow = 4
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n4", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["now"] < history[s$2]["current"]["header"]["time"]))

================================================================================
\* Created by Apalache on Thu Oct 22 13:13:09 CEST 2020
\* https://github.com/informalsystems/apalache