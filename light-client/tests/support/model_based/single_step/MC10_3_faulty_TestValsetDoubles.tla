------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n8", "n9" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5", "n7", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
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
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
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
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n8", "n9" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5", "n7", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> { "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n10", "n4" },
          VS |-> { "n8", "n9" },
          height |-> 4,
          lastCommit |-> { "n1", "n10", "n2", "n5", "n9" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
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
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n4" },
              VS |-> { "n8", "n9" },
              height |-> 4,
              lastCommit |-> { "n1", "n10", "n2", "n5", "n9" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1400
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n10", "n4" },
      VS |-> { "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n2", "n5", "n9" },
      time |-> 4]]
/\ prevNow = 1400
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State4 *)

State4 ==
/\ Faulty = { "n8", "n9" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n2", "n3", "n5", "n6", "n7", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5", "n7", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n3", "n5", "n7" },
      header |->
        [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
          VS |-> { "n3", "n5", "n7", "n8" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
          time |-> 2]]
  @@ 4
    :> [Commits |-> { "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n10", "n4" },
          VS |-> { "n8", "n9" },
          height |-> 4,
          lastCommit |-> { "n1", "n10", "n2", "n5", "n9" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
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
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n10", "n4" },
              VS |-> { "n8", "n9" },
              height |-> 4,
              lastCommit |-> { "n1", "n10", "n2", "n5", "n9" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n3", "n5", "n7" },
          header |->
            [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
              VS |-> { "n3", "n5", "n7", "n8" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
              time |-> 2]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n3", "n5", "n7", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n3", "n5", "n7" },
  header |->
    [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1400
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n3", "n5", "n7" },
  header |->
    [NextVS |-> { "n1", "n10", "n3", "n4", "n5", "n6", "n8" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4", "n5", "n6", "n8", "n9" },
      time |-> 2]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 3
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        s2$2 = s1$2 + 1
          /\ LET t_34 == history[s1$2]["current"]["header"]["VS"] IN
          BMC!Skolem((\E t_32 \in t_34:
            BMC!Skolem((\E t_33 \in t_34: ~(t_32 = t_33)))))
          /\ Cardinality(history[s2$2]["current"]["header"]["VS"])
            = 2 * Cardinality(history[s1$2]["current"]["header"]["VS"])))))

================================================================================
\* Created by Apalache on Fri Nov 06 10:06:21 UTC 2020
\* https://github.com/informalsystems/apalache
