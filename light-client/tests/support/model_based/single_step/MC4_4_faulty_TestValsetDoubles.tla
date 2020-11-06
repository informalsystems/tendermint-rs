------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]
  @@ 4
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n2"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1398
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]
  @@ 4
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n2"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1398
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n4"},
      height |-> 5,
      lastCommit |-> {},
      time |-> 4]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 2 to State4 *)

State4 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]
  @@ 4
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n2"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n2" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2" },
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1398
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n1", "n2" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n2" },
      VS |-> { "n1", "n2" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]
  @@ 4
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n2"},
      height |-> 5,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n2" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2" },
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
  @@ 4
    :> [Commits |-> { "n1", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 4,
          lastCommit |-> { "n1", "n2" },
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n4"},
          height |-> 5,
          lastCommit |-> {},
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n4"},
              height |-> 5,
              lastCommit |-> {},
              time |-> 4]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 4,
              lastCommit |-> { "n1", "n2" },
              time |-> 7]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]]]
/\ latestVerified = [Commits |-> { "n1", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateVerified" @@ 4 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1398
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n1", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n1", "n2" },
      time |-> 7]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 4
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        s2$2 = s1$2 + 1
          /\ LET t_34 == history[s1$2]["current"]["header"]["VS"] IN
          BMC!Skolem((\E t_32 \in t_34:
            BMC!Skolem((\E t_33 \in t_34: ~(t_32 = t_33)))))
          /\ Cardinality(history[s2$2]["current"]["header"]["VS"])
            = 2 * Cardinality(history[s1$2]["current"]["header"]["VS"])))))

================================================================================
\* Created by Apalache on Fri Nov 06 10:19:02 UTC 2020
\* https://github.com/informalsystems/apalache
