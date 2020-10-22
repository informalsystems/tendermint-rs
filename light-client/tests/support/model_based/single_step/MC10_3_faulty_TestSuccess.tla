------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n2", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n7", "n8" },
      height |-> 4,
      lastCommit |-> { "n1", "n10" },
      time |-> 5]
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
      now |-> 5,
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
/\ now = 5
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 5
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
/\ Faulty = { "n2", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n7", "n8" },
      height |-> 4,
      lastCommit |-> { "n1", "n10" },
      time |-> 5]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n3", "n5", "n7", "n8" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n1", "n10" },
      header |->
        [NextVS |-> { "n10", "n2", "n7", "n8" },
          VS |-> { "n1", "n10" },
          height |-> 3,
          lastCommit |-> { "n3", "n5", "n7", "n8" },
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
      now |-> 5,
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
        [Commits |-> { "n1", "n10" },
          header |->
            [NextVS |-> { "n10", "n2", "n7", "n8" },
              VS |-> { "n1", "n10" },
              height |-> 3,
              lastCommit |-> { "n3", "n5", "n7", "n8" },
              time |-> 4]],
      now |-> 5,
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
/\ now = 5
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n1", "n10" },
  header |->
    [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]]
/\ prevNow = 5
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State4 *)

State4 ==
/\ Faulty = { "n2", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n7", "n8" },
      height |-> 4,
      lastCommit |-> { "n1", "n10" },
      time |-> 5]
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
        [NextVS |-> { "n1", "n10" },
          VS |-> { "n3", "n5", "n7", "n8" },
          height |-> 2,
          lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
          time |-> 3]]
  @@ 3
    :> [Commits |-> { "n1", "n10" },
      header |->
        [NextVS |-> { "n10", "n2", "n7", "n8" },
          VS |-> { "n1", "n10" },
          height |-> 3,
          lastCommit |-> { "n3", "n5", "n7", "n8" },
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
      now |-> 5,
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
        [Commits |-> { "n1", "n10" },
          header |->
            [NextVS |-> { "n10", "n2", "n7", "n8" },
              VS |-> { "n1", "n10" },
              height |-> 3,
              lastCommit |-> { "n3", "n5", "n7", "n8" },
              time |-> 4]],
      now |-> 5,
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
            [NextVS |-> { "n1", "n10" },
              VS |-> { "n3", "n5", "n7", "n8" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
              time |-> 3]],
      now |-> 5,
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
    [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1402
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n3", "n5", "n7" },
  header |->
    [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]]
/\ prevNow = 5
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State5 *)

State5 ==
/\ Faulty = { "n2", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n3", "n5", "n7", "n8" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n10", "n2", "n7", "n8" },
      height |-> 4,
      lastCommit |-> { "n1", "n10" },
      time |-> 5]
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
        [NextVS |-> { "n1", "n10" },
          VS |-> { "n3", "n5", "n7", "n8" },
          height |-> 2,
          lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
          time |-> 3]]
  @@ 3
    :> [Commits |-> { "n1", "n10" },
      header |->
        [NextVS |-> { "n10", "n2", "n7", "n8" },
          VS |-> { "n1", "n10" },
          height |-> 3,
          lastCommit |-> { "n3", "n5", "n7", "n8" },
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
      now |-> 5,
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
        [Commits |-> { "n1", "n10" },
          header |->
            [NextVS |-> { "n10", "n2", "n7", "n8" },
              VS |-> { "n1", "n10" },
              height |-> 3,
              lastCommit |-> { "n3", "n5", "n7", "n8" },
              time |-> 4]],
      now |-> 5,
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
            [NextVS |-> { "n1", "n10" },
              VS |-> { "n3", "n5", "n7", "n8" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
              time |-> 3]],
      now |-> 5,
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
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n10" },
          header |->
            [NextVS |-> { "n10", "n2", "n7", "n8" },
              VS |-> { "n1", "n10" },
              height |-> 3,
              lastCommit |-> { "n3", "n5", "n7", "n8" },
              time |-> 4]],
      now |-> 1402,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n3", "n5", "n7" },
          header |->
            [NextVS |-> { "n1", "n10" },
              VS |-> { "n3", "n5", "n7", "n8" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
              time |-> 3]]]
/\ latestVerified = [Commits |-> { "n1", "n10" },
  header |->
    [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1402
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n1", "n10" },
  header |->
    [NextVS |-> { "n10", "n2", "n7", "n8" },
      VS |-> { "n1", "n10" },
      height |-> 3,
      lastCommit |-> { "n3", "n5", "n7", "n8" },
      time |-> 4]]
/\ prevNow = 1402
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n3", "n5", "n7" },
  header |->
    [NextVS |-> { "n1", "n10" },
      VS |-> { "n3", "n5", "n7", "n8" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7" },
      time |-> 3]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedSuccess" /\ Cardinality((DOMAIN fetchedLightBlocks)) = 3

================================================================================
\* Created by Apalache on Thu Oct 22 13:12:44 CEST 2020
\* https://github.com/informalsystems/apalache
