__scheduler.tla__

The specification of the fastsync v2 scheduler handler in:
 https://github.com/tendermint/tendermint/blockchain/v2/scheduler.go

Run with:
 - Constants:
    - numRequests <- 2
    - PeerIDs <- 0..2
    - ultimateHeight <- 3
 - Invariants:
    - TypeOK
 - Properties:
    - TerminationWhenNoAdvance
    - TerminationGoodPeers
    - TerminationAllCases
 - Proofs that properties are not vacuously true:
    - TerminationGoodPeersPre
    - TerminationAllCases
    - SchedulerIncreasePre

