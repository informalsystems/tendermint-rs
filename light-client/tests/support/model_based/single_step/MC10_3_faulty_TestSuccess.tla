------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n10", "n4", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n8" },
      height |-> 4,
      lastCommit |-> { "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1399
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1399
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n10", "n4", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n8" },
      height |-> 4,
      lastCommit |-> { "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
      header |->
        [NextVS |-> { "n3", "n4", "n6", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 3,
          lastCommit |-> { "n10", "n2", "n3", "n8" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n10", "n2", "n3", "n8" },
              time |-> 3]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1399
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
  header |->
    [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]]
/\ prevNow = 1399
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State4 *)

State4 ==
/\ Faulty = { "n10", "n4", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n8" },
      height |-> 4,
      lastCommit |-> { "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n2", "n3", "n8" },
      header |->
        [NextVS |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> { "n10", "n2", "n3", "n8" },
          height |-> 2,
          lastCommit |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
          time |-> 2]]
  @@ 3
    :> [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
      header |->
        [NextVS |-> { "n3", "n4", "n6", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 3,
          lastCommit |-> { "n10", "n2", "n3", "n8" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n10", "n2", "n3", "n8" },
              time |-> 3]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n2", "n3", "n8" },
          header |->
            [NextVS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n8" },
              height |-> 2,
              lastCommit |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
              time |-> 2]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n2", "n3", "n8" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1401
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n2", "n3", "n8" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]]
/\ prevNow = 1399
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State5 *)

State5 ==
/\ Faulty = { "n10", "n4", "n7" }
/\ blockchain = 1
    :> [NextVS |-> { "n10", "n2", "n3", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]
  @@ 3
    :> [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n3", "n4", "n6", "n8" },
      height |-> 4,
      lastCommit |-> { "n2", "n4", "n5", "n6", "n7", "n8", "n9" },
      time |-> 4]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n2", "n3", "n8" },
      header |->
        [NextVS |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          VS |-> { "n10", "n2", "n3", "n8" },
          height |-> 2,
          lastCommit |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
          time |-> 2]]
  @@ 3
    :> [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
      header |->
        [NextVS |-> { "n3", "n4", "n6", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 3,
          lastCommit |-> { "n10", "n2", "n3", "n8" },
          time |-> 3]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n10", "n2", "n3", "n8" },
              time |-> 3]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n2", "n3", "n8" },
          header |->
            [NextVS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n8" },
              height |-> 2,
              lastCommit |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
              time |-> 2]],
      now |-> 1399,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
          header |->
            [NextVS |-> { "n3", "n4", "n6", "n8" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 3,
              lastCommit |-> { "n10", "n2", "n3", "n8" },
              time |-> 3]],
      now |-> 1401,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n2", "n3", "n8" },
          header |->
            [NextVS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n8" },
              height |-> 2,
              lastCommit |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
              time |-> 2]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
  header |->
    [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1401
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n1", "n10", "n4", "n5", "n6", "n7", "n9" },
  header |->
    [NextVS |-> { "n3", "n4", "n6", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 3,
      lastCommit |-> { "n10", "n2", "n3", "n8" },
      time |-> 3]]
/\ prevNow = 1401
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n2", "n3", "n8" },
  header |->
    [NextVS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8" },
      time |-> 2]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedSuccess" /\ Cardinality((DOMAIN fetchedLightBlocks)) = 3

================================================================================
\* Created by Apalache on Thu Oct 15 12:40:36 CEST 2020
\* https://github.com/informalsystems/apalache
