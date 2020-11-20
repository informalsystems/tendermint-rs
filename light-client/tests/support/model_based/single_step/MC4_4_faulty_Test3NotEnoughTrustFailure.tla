------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1396
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1396
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1397
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> {"n1"},
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 4]]
/\ prevNow = 1396
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 0 to State4 *)

State4 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
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
    [NextVS |-> {"n1"},
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 4]]
/\ prevNow = 1397
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n2"},
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 6]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 4]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1398
/\ nprobes = 3
/\ prevCurrent = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 4 to State6 *)

State6 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n2"},
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 6]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n1"},
          height |-> 4,
          lastCommit |-> {"n2"},
          time |-> 8]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n1"},
              height |-> 4,
              lastCommit |-> {"n2"},
              time |-> 8]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1406
/\ nprobes = 4
/\ prevCurrent = [Commits |-> {"n1"},
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ state = "working"

(* Transition 2 to State7 *)

State7 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]
  @@ 3
    :> [NextVS |-> {"n1"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> {"n2"},
      time |-> 7]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n1"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 8]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n1"},
      time |-> 9]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n2"},
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 6]]
  @@ 3
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n3"},
          height |-> 3,
          lastCommit |-> { "n1", "n2" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n1"},
          height |-> 4,
          lastCommit |-> {"n2"},
          time |-> 8]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1396,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1396,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1397,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n1"},
              height |-> 4,
              lastCommit |-> {"n2"},
              time |-> 8]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]]]
  @@ 5
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n3"},
              height |-> 3,
              lastCommit |-> { "n1", "n2" },
              time |-> 4]],
      now |-> 1406,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n2"},
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 6]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateFailed"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1406
/\ nprobes = 5
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> {"n1"},
      VS |-> {"n3"},
      height |-> 3,
      lastCommit |-> { "n1", "n2" },
      time |-> 4]]
/\ prevNow = 1406
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n2"},
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 6]]
/\ state = "finishedFailure"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedFailure"
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        BMC!Skolem((\E s3$2 \in DOMAIN history:
          ((~(s1$2 = s2$2) /\ ~(s2$2 = s3$2)) /\ ~(s1$2 = s3$2))
            /\ history[s1$2]["verdict"] = "NOT_ENOUGH_TRUST"
            /\ history[s2$2]["verdict"] = "NOT_ENOUGH_TRUST"
            /\ history[s3$2]["verdict"] = "NOT_ENOUGH_TRUST"))))))

================================================================================
\* Created by Apalache on Wed Nov 18 12:38:49 UTC 2020
\* https://github.com/informalsystems/apalache
