------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> { "n3", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
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
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> { "n3", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> { "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n3", "n4" },
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n3", "n4" },
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1400
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 4]]
/\ prevNow = 1400
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 2 to State4 *)

State4 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> { "n3", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n2", "n4" },
      header |->
        [NextVS |-> { "n2", "n3" },
          VS |-> { "n1", "n2", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 2]]
  @@ 4
    :> [Commits |-> { "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n3", "n4" },
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n3", "n4" },
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n1", "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 2]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1401
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n1", "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State5 *)

State5 ==
/\ Faulty = {}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> { "n3", "n4" },
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 4]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> { "n3", "n4" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n2", "n4" },
      header |->
        [NextVS |-> { "n2", "n3" },
          VS |-> { "n1", "n2", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 2]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> { "n3", "n4" },
          VS |-> { "n2", "n3" },
          height |-> 3,
          lastCommit |-> { "n1", "n2", "n4" },
          time |-> 3]]
  @@ 4
    :> [Commits |-> { "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> { "n3", "n4" },
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n3", "n4" },
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 4]],
      now |-> 1400,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n1", "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 2]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> { "n3", "n4" },
              VS |-> { "n2", "n3" },
              height |-> 3,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 3]],
      now |-> 1401,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> { "n1", "n2", "n4" },
          header |->
            [NextVS |-> { "n2", "n3" },
              VS |-> { "n1", "n2", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n4" },
              time |-> 2]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateFailed"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1401
/\ nprobes = 3
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> { "n3", "n4" },
      VS |-> { "n2", "n3" },
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 3]]
/\ prevNow = 1401
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n2", "n4" },
  header |->
    [NextVS |-> { "n2", "n3" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n4" },
      time |-> 2]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 4
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["verified"]["header"]["time"] + 1400 > history[s$2]["now"]
        /\ BMC!Skolem((\E commits$2 \in SUBSET ({ "n1", "n2", "n3", "n4" }):
          BMC!Skolem((\E t_39$1 \in SUBSET ({ "n1", "n2", "n3", "n4" }):
            BMC!Skolem((\E t_37$1 \in SUBSET ({ "n1", "n2", "n3", "n4" }):
              BMC!Skolem((\E t_36$1 \in SUBSET ({ "n1", "n2", "n3", "n4" }):
                BMC!Skolem((\E t_35$1 \in SUBSET ({ "n1", "n2", "n3", "n4" }):
                  BMC!Skolem((\E t_34$1 \in Int:
                    BMC!Skolem((\E t_33$1 \in 1 .. 5:
                      LET t_3n ==
                        [height |-> t_33$1,
                          time |-> t_34$1,
                          lastCommit |-> t_35$1,
                          VS |-> t_36$1,
                          NextVS |-> t_37$1]
                      IN
                      LET t_3m == [header |-> t_3n, Commits |-> t_39$1] IN
                      3 * Cardinality(commits$2)
                          < 2
                            * Cardinality(history[s$2]["current"]["header"][
                              "VS"
                            ])
                        /\ LET ref$5 ==
                          blockchain[
                            history[s$2]["current"]["header"]["height"]
                          ]
                        IN
                        LET lastCommit$6 ==
                          IF history[s$2]["current"]["header"]["height"] < 5
                          THEN blockchain[
                            (history[s$2]["current"]["header"]["height"] + 1)
                          ][
                            "lastCommit"
                          ]
                          ELSE blockchain[
                            history[s$2]["current"]["header"]["height"]
                          ][
                            "VS"
                          ]
                        IN
                        t_3m = [header |-> ref$5, Commits |-> lastCommit$6]
                        /\ history[s$2]["current"]
                          = [ (t_3m) EXCEPT ![<<"Commits">>] = commits$2 ]))))))))))))))))

================================================================================
\* Created by Apalache on Wed Nov 18 12:42:35 UTC 2020
\* https://github.com/informalsystems/apalache
