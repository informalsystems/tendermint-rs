------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
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
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 1398
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n4"},
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 11]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1399
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n3"},
      VS |-> {"n4"},
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 11]]
/\ prevNow = 1398
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State4 *)

State4 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 3
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n4"},
          height |-> 3,
          lastCommit |-> {"n3"},
          time |-> 6]]
  @@ 4
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n4"},
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 11]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n3"},
              time |-> 6]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 1400
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n1"},
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n3"},
      time |-> 6]]
/\ prevNow = 1399
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 8]]
  @@ 3
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n4"},
          height |-> 3,
          lastCommit |-> {"n3"},
          time |-> 6]]
  @@ 4
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n4"},
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 11]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n3"},
              time |-> 6]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1405
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
/\ prevNow = 1400
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State6 *)

State6 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 8]]
  @@ 3
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n4"},
          height |-> 3,
          lastCommit |-> {"n3"},
          time |-> 6]]
  @@ 4
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n4"},
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 11]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n3"},
              time |-> 6]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1405,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1405
/\ nprobes = 4
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n3"},
      VS |-> {"n4"},
      height |-> 4,
      lastCommit |-> { "n2", "n3" },
      time |-> 11]]
/\ prevNow = 1405
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
/\ state = "working"

(* Transition 2 to State7 *)

State7 ==
/\ Faulty = {"n4"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]
  @@ 3
    :> [NextVS |-> {"n3"},
      VS |-> {"n2"},
      height |-> 3,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 10]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> {"n3"},
      height |-> 4,
      lastCommit |-> {"n2"},
      time |-> 11]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 5,
      lastCommit |-> {"n3"},
      time |-> 12]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n2", "n3", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> {"n2"},
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n3", "n4" },
          time |-> 8]]
  @@ 3
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n1"},
          VS |-> {"n4"},
          height |-> 3,
          lastCommit |-> {"n3"},
          time |-> 6]]
  @@ 4
    :> [Commits |-> {"n4"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n4"},
          height |-> 4,
          lastCommit |-> { "n2", "n3" },
          time |-> 11]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 1398,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1398,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n3"},
              time |-> 6]],
      now |-> 1399,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]],
      now |-> 1400,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n2", "n3", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n4"},
              height |-> 4,
              lastCommit |-> { "n2", "n3" },
              time |-> 11]],
      now |-> 1405,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]]]
  @@ 5
    :> [current |->
        [Commits |-> {"n4"},
          header |->
            [NextVS |-> {"n1"},
              VS |-> {"n4"},
              height |-> 3,
              lastCommit |-> {"n3"},
              time |-> 6]],
      now |-> 1405,
      verdict |-> "INVALID",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> {"n2"},
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n3", "n4" },
              time |-> 8]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateFailed"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 1405
/\ nprobes = 5
/\ prevCurrent = [Commits |-> {"n4"},
  header |->
    [NextVS |-> {"n1"},
      VS |-> {"n4"},
      height |-> 3,
      lastCommit |-> {"n3"},
      time |-> 6]]
/\ prevNow = 1405
/\ prevVerdict = "INVALID"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> {"n2"},
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n3", "n4" },
      time |-> 8]]
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
\* Created by Apalache on Fri Nov 06 10:12:47 UTC 2020
\* https://github.com/informalsystems/apalache
