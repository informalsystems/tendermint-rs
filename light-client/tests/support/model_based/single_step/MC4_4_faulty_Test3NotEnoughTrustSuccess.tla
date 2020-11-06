------------------------- MODULE counterexample -------------------------

EXTENDS MC4_4_faulty

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified"
/\ nextHeight = 4
/\ now = 8
/\ nprobes = 0
/\ prevCurrent = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ prevNow = 8
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State3 *)

State3 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 8
/\ nprobes = 1
/\ prevCurrent = [Commits |-> {"n2"},
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]]
/\ prevNow = 8
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State4 *)

State4 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n3"},
          height |-> 5,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n3"},
              height |-> 5,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ lightBlockStatus = 1 :> "StateVerified" @@ 3 :> "StateUnverified" @@ 4 :> "StateUnverified"
/\ nextHeight = 2
/\ now = 9
/\ nprobes = 2
/\ prevCurrent = [Commits |-> {"n3"},
  header |->
    [NextVS |-> {"n3"},
      VS |-> {"n3"},
      height |-> 5,
      lastCommit |-> { "n1", "n2" },
      time |-> 5]]
/\ prevNow = 8
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 3 to State5 *)

State5 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n4" },
      header |->
        [NextVS |-> {"n1"},
          VS |-> { "n1", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n3" },
          time |-> 3]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n3"},
          height |-> 5,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n3"},
              height |-> 5,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
/\ latestVerified = [Commits |-> { "n1", "n4" },
  header |->
    [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 9
/\ nprobes = 3
/\ prevCurrent = [Commits |-> { "n1", "n4" },
  header |->
    [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]]
/\ prevNow = 9
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n2", "n3", "n4" },
  header |->
    [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]]
/\ state = "working"

(* Transition 1 to State6 *)

State6 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n4" },
      header |->
        [NextVS |-> {"n1"},
          VS |-> { "n1", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n3" },
          time |-> 3]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n3"},
          height |-> 5,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n3"},
              height |-> 5,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 9,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]]]
/\ latestVerified = [Commits |-> { "n1", "n4" },
  header |->
    [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateUnverified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 3
/\ now = 9
/\ nprobes = 4
/\ prevCurrent = [Commits |-> {"n2"},
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]]
/\ prevNow = 9
/\ prevVerdict = "NOT_ENOUGH_TRUST"
/\ prevVerified = [Commits |-> { "n1", "n4" },
  header |->
    [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]]
/\ state = "working"

(* Transition 2 to State7 *)

State7 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n4" },
      header |->
        [NextVS |-> {"n1"},
          VS |-> { "n1", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n3" },
          time |-> 3]]
  @@ 3
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n1"},
          height |-> 3,
          lastCommit |-> { "n1", "n4" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n3"},
          height |-> 5,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n3"},
              height |-> 5,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 9,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]]]
  @@ 5
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n1"},
              height |-> 3,
              lastCommit |-> { "n1", "n4" },
              time |-> 4]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]]]
/\ latestVerified = [Commits |-> {"n1"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateVerified"
  @@ 4 :> "StateUnverified"
/\ nextHeight = 4
/\ now = 1403
/\ nprobes = 5
/\ prevCurrent = [Commits |-> {"n1"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]]
/\ prevNow = 9
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> { "n1", "n4" },
  header |->
    [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]]
/\ state = "working"

(* Transition 0 to State8 *)

State8 ==
/\ Faulty = {"n3"}
/\ blockchain = 1
    :> [NextVS |-> { "n1", "n4" },
      VS |-> { "n1", "n2", "n3", "n4" },
      height |-> 1,
      lastCommit |-> {},
      time |-> 1]
  @@ 2
    :> [NextVS |-> {"n1"},
      VS |-> { "n1", "n4" },
      height |-> 2,
      lastCommit |-> { "n1", "n2", "n3" },
      time |-> 3]
  @@ 3
    :> [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]
  @@ 4
    :> [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]
  @@ 5
    :> [NextVS |-> { "n1", "n2", "n3", "n4" },
      VS |-> { "n1", "n2", "n4" },
      height |-> 5,
      lastCommit |-> {"n2"},
      time |-> 8]
/\ fetchedLightBlocks = 1
    :> [Commits |-> { "n1", "n2", "n3", "n4" },
      header |->
        [NextVS |-> { "n1", "n4" },
          VS |-> { "n1", "n2", "n3", "n4" },
          height |-> 1,
          lastCommit |-> {},
          time |-> 1]]
  @@ 2
    :> [Commits |-> { "n1", "n4" },
      header |->
        [NextVS |-> {"n1"},
          VS |-> { "n1", "n4" },
          height |-> 2,
          lastCommit |-> { "n1", "n2", "n3" },
          time |-> 3]]
  @@ 3
    :> [Commits |-> {"n1"},
      header |->
        [NextVS |-> {"n2"},
          VS |-> {"n1"},
          height |-> 3,
          lastCommit |-> { "n1", "n4" },
          time |-> 4]]
  @@ 4
    :> [Commits |-> {"n2"},
      header |->
        [NextVS |-> { "n1", "n2", "n4" },
          VS |-> {"n2"},
          height |-> 4,
          lastCommit |-> {"n1"},
          time |-> 7]]
  @@ 5
    :> [Commits |-> {"n3"},
      header |->
        [NextVS |-> {"n3"},
          VS |-> {"n3"},
          height |-> 5,
          lastCommit |-> { "n1", "n2" },
          time |-> 5]]
/\ history = 0
    :> [current |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]],
      now |-> 8,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 1
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 2
    :> [current |->
        [Commits |-> {"n3"},
          header |->
            [NextVS |-> {"n3"},
              VS |-> {"n3"},
              height |-> 5,
              lastCommit |-> { "n1", "n2" },
              time |-> 5]],
      now |-> 8,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 3
    :> [current |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n2", "n3", "n4" },
          header |->
            [NextVS |-> { "n1", "n4" },
              VS |-> { "n1", "n2", "n3", "n4" },
              height |-> 1,
              lastCommit |-> {},
              time |-> 1]]]
  @@ 4
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 9,
      verdict |-> "NOT_ENOUGH_TRUST",
      verified |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]]]
  @@ 5
    :> [current |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n1"},
              height |-> 3,
              lastCommit |-> { "n1", "n4" },
              time |-> 4]],
      now |-> 9,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> { "n1", "n4" },
          header |->
            [NextVS |-> {"n1"},
              VS |-> { "n1", "n4" },
              height |-> 2,
              lastCommit |-> { "n1", "n2", "n3" },
              time |-> 3]]]
  @@ 6
    :> [current |->
        [Commits |-> {"n2"},
          header |->
            [NextVS |-> { "n1", "n2", "n4" },
              VS |-> {"n2"},
              height |-> 4,
              lastCommit |-> {"n1"},
              time |-> 7]],
      now |-> 1403,
      verdict |-> "SUCCESS",
      verified |->
        [Commits |-> {"n1"},
          header |->
            [NextVS |-> {"n2"},
              VS |-> {"n1"},
              height |-> 3,
              lastCommit |-> { "n1", "n4" },
              time |-> 4]]]
/\ latestVerified = [Commits |-> {"n2"},
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]]
/\ lightBlockStatus = 1 :> "StateVerified"
  @@ 2 :> "StateVerified"
  @@ 3 :> "StateVerified"
  @@ 4 :> "StateVerified"
/\ nextHeight = 4
/\ now = 1403
/\ nprobes = 6
/\ prevCurrent = [Commits |-> {"n2"},
  header |->
    [NextVS |-> { "n1", "n2", "n4" },
      VS |-> {"n2"},
      height |-> 4,
      lastCommit |-> {"n1"},
      time |-> 7]]
/\ prevNow = 1403
/\ prevVerdict = "SUCCESS"
/\ prevVerified = [Commits |-> {"n1"},
  header |->
    [NextVS |-> {"n2"},
      VS |-> {"n1"},
      height |-> 3,
      lastCommit |-> { "n1", "n4" },
      time |-> 4]]
/\ state = "finishedSuccess"

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  state = "finishedSuccess"
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        BMC!Skolem((\E s3$2 \in DOMAIN history:
          ((~(s1$2 = s2$2) /\ ~(s2$2 = s3$2)) /\ ~(s1$2 = s3$2))
            /\ history[s1$2]["verdict"] = "NOT_ENOUGH_TRUST"
            /\ history[s2$2]["verdict"] = "NOT_ENOUGH_TRUST"
            /\ history[s3$2]["verdict"] = "NOT_ENOUGH_TRUST"))))))

================================================================================
\* Created by Apalache on Fri Nov 06 10:12:07 UTC 2020
\* https://github.com/informalsystems/apalache
