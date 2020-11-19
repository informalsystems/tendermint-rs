------------------------- MODULE counterexample -------------------------

EXTENDS MC10_3_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = { "n5", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]
  @@ 3
    :> [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n3", "n4", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n7" },
      time |-> 1398]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 3
/\ now = 1398
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = { "n5", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]
  @@ 3
    :> [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n3", "n4", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n7" },
      time |-> 1398]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> { "n10", "n2", "n7" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
          VS |-> { "n10", "n2", "n3", "n7" },
          height |-> 3,
          lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
          time |-> 1397]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n10", "n2", "n7" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n7" },
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
              time |-> 1397]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1398
/\ nprobes = 1
/\ prevCurrent = [Commits |-> { "n10", "n2", "n7" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State4 *)

State4 ==
/\ Faulty = { "n5", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]
  @@ 3
    :> [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n3", "n4", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n7" },
      time |-> 1398]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n7" },
          VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          height |-> 2,
          lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
          time |-> 1396]]
  @@ 3
    :> [Commits |-> { "n10", "n2", "n7" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
          VS |-> { "n10", "n2", "n3", "n7" },
          height |-> 3,
          lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
          time |-> 1397]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n10", "n2", "n7" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n7" },
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
              time |-> 1397]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n7" },
              VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
              time |-> 1396]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 2795
/\ nprobes = 2
/\ prevCurrent = [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
  header |->
    [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State5 *)

State5 ==
/\ Faulty = { "n5", "n6" }
/\ blockchain = 1
    :> [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]
  @@ 3
    :> [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]
  @@ 4
    :> [NextVS |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      VS |-> { "n1", "n3", "n4", "n8", "n9" },
      height |-> 4,
      lastCommit |-> { "n10", "n3", "n7" },
      time |-> 1398]
/\ fetchedLightBlocks = 1
    :> [Commits |->
        { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
      header |->
        [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          VS |-> { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
      header |->
        [NextVS |-> { "n10", "n2", "n3", "n7" },
          VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
          height |-> 2,
          lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
          time |-> 1396]]
  @@ 3
    :> [Commits |-> { "n10", "n2", "n7" },
      header |->
        [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
          VS |-> { "n10", "n2", "n3", "n7" },
          height |-> 3,
          lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
          time |-> 1397]]
/\ history = 0
    :> [current |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> { "n10", "n2", "n7" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n7" },
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
              time |-> 1397]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n7" },
              VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
              time |-> 1396]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |->
            { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
          header |->
            [NextVS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              VS |->
                { "n1", "n10", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n10", "n2", "n7" },
          header |->
            [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
              VS |-> { "n10", "n2", "n3", "n7" },
              height |-> 3,
              lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
              time |-> 1397]],
      now |-> 2795,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
          header |->
            [NextVS |-> { "n10", "n2", "n3", "n7" },
              VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
              height |-> 2,
              lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
              time |-> 1396]]]
/\ latestVerified = [Commits |-> { "n10", "n2", "n7" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 2 :> "StateVerified" @@ 3 :> "StateVerified"
/\ nextHeight = 3
/\ now = 2795
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n10", "n2", "n7" },
  header |->
    [NextVS |-> { "n1", "n3", "n4", "n8", "n9" },
      VS |-> { "n10", "n2", "n3", "n7" },
      height |-> 3,
      lastCommit |-> { "n2", "n3", "n6", "n8", "n9" },
      time |-> 1397]]
/\ prevNow = 2795
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n3", "n4", "n5", "n6", "n9" },
  header |->
    [NextVS |-> { "n10", "n2", "n3", "n7" },
      VS |-> { "n2", "n3", "n4", "n5", "n6", "n8", "n9" },
      height |-> 2,
      lastCommit |-> { "n1", "n10", "n2", "n4", "n5", "n6", "n7", "n9" },
      time |-> 1396]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedSuccess" /\ Cardinality((DOMAIN fetchedLightBlocks)) = 3

================================================================================
\* Created by Apalache on Wed Nov 18 13:55:05 UTC 2020
\* https://github.com/informalsystems/apalache
