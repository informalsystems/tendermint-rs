------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n1", "n7", "n8" }
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n6", "n8", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n5", "n8" },
      VS |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n6", "n9" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n5", "n8" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1400
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State3 *)

State3 ==
/\ Faulty = { "n1", "n7", "n8" }
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n6", "n8", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n5", "n8" },
      VS |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n6", "n9" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n5", "n8" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n4", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> { "n1", "n7", "n8" },
      header |->
        [NextVS |-> {"n7"},
          VS |-> { "n2", "n4", "n8", "n9" },
          height |-> 4,
          lastCommit |->
            { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
          time |-> 0]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n7", "n8" },
          header |->
            [NextVS |-> {"n7"},
              VS |-> { "n2", "n4", "n8", "n9" },
              height |-> 4,
              lastCommit |->
                { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
              time |-> 0]],
      now |-> 1400,
      verdict |-> "INVALID",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateFailed"
/\ nextHeight = 3
/\ now = 1400
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n7", "n8" },
  header |->
    [NextVS |-> {"n7"},
      VS |-> { "n2", "n4", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
      time |-> 0]]
/\ prevNow = 1400
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  BMC!Skolem((\E s$2 \in DOMAIN history:
      LET CMS$2 == history[s$2]["current"]["Commits"] IN
      LET UVS$2 == history[s$2]["current"]["header"]["VS"] IN
      history[s$2]["current"]["header"]["time"]
          < history[s$2]["verified"]["header"]["time"]
        /\ history[s$2]["now"]
          < history[s$2]["verified"]["header"]["time"] + 1400
        /\ ~(CMS$2 = {} <: {STRING})
        /\ ~(UVS$2 = {} <: {STRING})
        /\ Cardinality((CMS$2)) < Cardinality((UVS$2))))

================================================================================
\* Created by Apalache on Fri Nov 06 10:04:10 UTC 2020
\* https://github.com/informalsystems/apalache
