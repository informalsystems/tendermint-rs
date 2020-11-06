------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n4", "n7", "n9" },
      VS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n10"},
      VS |-> { "n10", "n4", "n7", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> {"n10"},
      height |-> 4,
      lastCommit |-> { "n10", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1397,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1397
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1397
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n4", "n7", "n9" },
      VS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n10"},
      VS |-> { "n10", "n4", "n7", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> {"n10"},
      height |-> 4,
      lastCommit |-> { "n10", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n10", "n4", "n9" },
      header |->
        [NextVS |-> {"n10"},
          VS |-> { "n10", "n4", "n7", "n9" },
          height |-> 3,
          lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1397,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n10", "n4", "n9" },
          header |->
            [NextVS |-> {"n10"},
              VS |-> { "n10", "n4", "n7", "n9" },
              height |-> 3,
              lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1400
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n10", "n4", "n9" },
  header |->
    [NextVS |-> {"n10"},
      VS |-> { "n10", "n4", "n7", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
      time |-> 4]]
/\ prevNow = 1397
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 5 to State4 *)

State4 ==
/\ Faulty = { "n3", "n4" }
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n4", "n7", "n9" },
      VS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> {"n10"},
      VS |-> { "n10", "n4", "n7", "n9" },
      height |-> 3,
      lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> {"n10"},
      height |-> 4,
      lastCommit |-> { "n10", "n7", "n9" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n10", "n4", "n9" },
      header |->
        [NextVS |-> {"n10"},
          VS |-> { "n10", "n4", "n7", "n9" },
          height |-> 3,
          lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {},
      header |->
        [NextVS |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> {"n10"},
          height |-> 4,
          lastCommit |-> { "n10", "n7", "n9" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1397,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n10", "n4", "n9" },
          header |->
            [NextVS |-> {"n10"},
              VS |-> { "n10", "n4", "n7", "n9" },
              height |-> 3,
              lastCommit |-> { "n3", "n4", "n5", "n7", "n8", "n9" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {},
          header |->
            [NextVS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |-> {"n10"},
              height |-> 4,
              lastCommit |-> { "n10", "n7", "n9" },
              time |-> 5]],
      now |-> 1400,
      verdict |-> "INVALID",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateFailed" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1400
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {},
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> {"n10"},
      height |-> 4,
      lastCommit |-> { "n10", "n7", "n9" },
      time |-> 5]]
/\ prevNow = 1400
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  Cardinality((DOMAIN fetchedLightBlocks)) = 3
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      history[s$2]["verified"]["header"]["time"] + 1400 > history[s$2]["now"]
        /\ BMC!Skolem((\E commits$2 \in SUBSET ({ "n1",
          "n2",
          "n3",
          "n4",
          "n5",
          "n6",
          "n7",
          "n8",
          "n9",
          "n10" }):
          BMC!Skolem((\E t_3a$1 \in SUBSET ({ "n1",
            "n2",
            "n3",
            "n4",
            "n5",
            "n6",
            "n7",
            "n8",
            "n9",
            "n10" }):
            BMC!Skolem((\E t_38$1 \in SUBSET ({ "n1",
              "n2",
              "n3",
              "n4",
              "n5",
              "n6",
              "n7",
              "n8",
              "n9",
              "n10" }):
              BMC!Skolem((\E t_37$1 \in SUBSET ({ "n1",
                "n2",
                "n3",
                "n4",
                "n5",
                "n6",
                "n7",
                "n8",
                "n9",
                "n10" }):
                BMC!Skolem((\E t_36$1 \in SUBSET ({ "n1",
                  "n2",
                  "n3",
                  "n4",
                  "n5",
                  "n6",
                  "n7",
                  "n8",
                  "n9",
                  "n10" }):
                  BMC!Skolem((\E t_35$1 \in Int:
                    BMC!Skolem((\E t_34$1 \in 1 .. 4:
                      LET t_3o ==
                        [height |-> t_34$1,
                          time |-> t_35$1,
                          lastCommit |-> t_36$1,
                          VS |-> t_37$1,
                          NextVS |-> t_38$1]
                      IN
                      LET t_3n == [header |-> t_3o, Commits |-> t_3a$1] IN
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
                          IF history[s$2]["current"]["header"]["height"] < 4
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
                        t_3n = [header |-> ref$5, Commits |-> lastCommit$6]
                        /\ history[s$2]["current"]
                          = [ (t_3n) EXCEPT ![<<"Commits">>] = commits$2 ]))))))))))))))))

================================================================================
\* Created by Apalache on Fri Nov 06 10:09:13 UTC 2020
\* https://github.com/informalsystems/apalache
