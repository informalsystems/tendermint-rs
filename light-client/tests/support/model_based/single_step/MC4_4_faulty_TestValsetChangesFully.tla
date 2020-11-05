------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2" },
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 4,
      lastCommit |-> {"n3"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n3", "n4" },
      height |-> 5,
      lastCommit |-> { "n1", "n2" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
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
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2" },
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 4,
      lastCommit |-> {"n3"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n3", "n4" },
      height |-> 5,
      lastCommit |-> { "n1", "n2" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> { "n1", "n2" },
      header |->
        [NextVS |-> { "n1", "n3", "n4" },
          VS |-> { "n1", "n2" },
          height |-> 4,
          lastCommit |-> {"n3"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 4,
              lastCommit |-> {"n3"},
              time |-> 5]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1398
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n2" },
  header |->
    [NextVS |-> { "n1", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 4,
      lastCommit |-> {"n3"},
      time |-> 5]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State4 *)

State4 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2" },
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 4,
      lastCommit |-> {"n3"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n3", "n4" },
      height |-> 5,
      lastCommit |-> { "n1", "n2" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> { "n1", "n2" },
      header |->
        [NextVS |-> { "n1", "n3", "n4" },
          VS |-> { "n1", "n2" },
          height |-> 4,
          lastCommit |-> {"n3"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 4,
              lastCommit |-> {"n3"},
              time |-> 5]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 4]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1398
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> { "n1", "n2" },
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n1", "n2" },
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n3", "n4" },
      VS |-> { "n1", "n2" },
      height |-> 4,
      lastCommit |-> {"n3"},
      time |-> 5]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n3", "n4" },
      height |-> 5,
      lastCommit |-> { "n1", "n2" },
      time |-> 6]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n3"},
          VS |-> { "n2", "n3", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n3" },
          time |-> 2]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> { "n1", "n2" },
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n2", "n3", "n4" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> { "n1", "n2" },
      header |->
        [NextVS |-> { "n1", "n3", "n4" },
          VS |-> { "n1", "n2" },
          height |-> 4,
          lastCommit |-> {"n3"},
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n2" },
          header |->
            [NextVS |-> { "n1", "n3", "n4" },
              VS |-> { "n1", "n2" },
              height |-> 4,
              lastCommit |-> {"n3"},
              time |-> 5]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> { "n1", "n2" },
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n4" },
              time |-> 4]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n3"},
              VS |-> { "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 2]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1398
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n3"},
      VS |-> { "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 2]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 4
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        s2$2 = s1$2 + 1
          /\ LET t_2z == history[s1$2]["current"]["header"]["VS"] IN
          BMC!Skolem((\E t_2x \in t_2z:
            BMC!Skolem((\E t_2y \in t_2z: ~(t_2x = t_2y)))))
          /\ {
            t_2k$1 \in history[s1$2]["current"]["header"]["VS"]:
              t_2k$1 \in history[s2$2]["current"]["header"]["VS"]
          }
            = {} <: {STRING}))))

================================================================================
\* Created by Apalache on Thu Oct 22 13:27:54 CEST 2020
\* https://github.com/informalsystems/apalache