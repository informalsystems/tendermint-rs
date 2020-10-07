------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 3,
      lastCommit |-> { "n2", "n3" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1401,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1401
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1401
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State3 *)

State3 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 3,
      lastCommit |-> { "n2", "n3" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> {"n4"},
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> {},
      header |->
        [NextVS |-> {"n4"},
          VS |-> {},
          height |-> 4,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 1402]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1401,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {},
          header |->
            [NextVS |-> {"n4"},
              VS |-> {},
              height |-> 4,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 1402]],
      now |-> 1401,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateFailed"
/\ nextHeight = 4
/\ now = 1401
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {},
  header |->
    [NextVS |-> {"n4"},
      VS |-> {},
      height |-> 4,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 1402]]
/\ prevNow = 1401
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["now"] < history[s$2]["current"]["header"]["time"]))

================================================================================
\* Created by Apalache on Wed Oct 07 14:21:09 UTC 2020
\* https://github.com/informalsystems/apalache
