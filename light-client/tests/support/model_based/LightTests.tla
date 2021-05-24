------------------------- MODULE LightTests ---------------------------

EXTENDS Lightclient_003_draft

(* The light client history, which is the function mapping states 0..nprobes to the record with fields:
   - verified: the latest verified block in the previous state
   - current: the block that is being checked in the previous state
   - now: the time point in the previous state
   - verdict: the light client verdict in the previous state
*)
VARIABLE
  \* @type: Int -> [verified: BLOCK, current: BLOCK, now: Int, verdict: Str];
  history

\* This predicate extends the LightClient Init predicate with history tracking
InitTest ==
  /\ Init
  /\ history = [ n \in {0} |->
     [ verified |-> prevVerified, current |-> prevCurrent, now |-> prevNow, verdict |-> prevVerdict ]]

\* This predicate extends the LightClient Next predicate with history tracking
NextTest ==
  /\ Next
  /\ history' = [ n \in DOMAIN history \union {nprobes'} |->
       IF n = nprobes' THEN
         [ verified |-> prevVerified', current |-> prevCurrent', now |-> prevNow', verdict |-> prevVerdict' ]
       ELSE history[n]
     ]

\* Some useful operators for writing tests

LightBlock(st) == history[st].current
Height(st) == history[st].current.header.height
ValSet(st) == LightBlock(st).header.VS
ValCommits(st) == LightBlock(st).Commits

\* Test an execution that finishes with failure
TestFailure ==
    /\ state = "finishedFailure"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

\* Test an execution that finishes with success
TestSuccess ==
    /\ state = "finishedSuccess"
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT

\* This test never produces a counterexample; so the model should be corrected
TestFailedTrustingPeriod ==
   \E s \in DOMAIN history :
      history[s].verdict = "FAILED_TRUSTING_PERIOD"

TwoNotEnoughTrust ==
   \E s1, s2 \in DOMAIN history :
       /\ s1 /= s2
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"

ThreeNotEnoughTrust ==
  \E s1, s2, s3 \in DOMAIN history :
       /\ s1 /= s2 /\ s2 /= s3 /\ s1 /= s3
       /\ history[s1].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
       /\ history[s3].verdict = "NOT_ENOUGH_TRUST"

\* Test an execution that finishes with success, and processes two headers with insufficient trust on the way
Test2NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ TwoNotEnoughTrust

\* Test an execution that finishes with failure, and processes two headers with insufficient trust on the way
Test2NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ TwoNotEnoughTrust


\* Test an execution that finishes with success, and processes three headers with insufficient trust on the way
Test3NotEnoughTrustSuccess ==
    /\ state = "finishedSuccess"
    /\ ThreeNotEnoughTrust

\* Test an execution that finishes with failure, and processes three headers with insufficient trust on the way
Test3NotEnoughTrustFailure ==
    /\ state = "finishedFailure"
    /\ ThreeNotEnoughTrust

TestNonMonotonicHeight ==
    /\ \E s \in DOMAIN history :
       \* this is wrong
       /\ history[s].current.header.height <= history[s].verified.header.height
       \* everything else is correct
       /\ history[s].current.header /= history[s].verified.header
       /\ history[s].current.header.time > history[s].verified.header.time
       /\ history[s].current.header.time < history[s].now
       /\ history[s].verified.header.time + TRUSTING_PERIOD > history[s].now
       /\ history[s].current.Commits /= {}
       /\ history[s].current.Commits \subseteq  history[s].current.header.VS

(*
Following three tests have been disabled for now,
please refer issue: https://github.com/informalsystems/tendermint-rs/issues/659
TestEmptyCommitEmptyValset ==
    /\ \E s \in DOMAIN history :
       \* this is wrong
       /\ history[s].current.Commits = {}
       /\ ValSet(s) = {}
       \* everything else is correct
       /\ history[s].current.header /= history[s].verified.header
       /\ history[s].current.header.height > history[s].verified.header.height
       /\ history[s].current.header.time > history[s].verified.header.time
       /\ history[s].current.header.time < history[s].now
       /\ history[s].verified.header.time + TRUSTING_PERIOD > history[s].now

TestEmptyCommitNonEmptyValset ==
    /\ \E s \in DOMAIN history :
       \* this is wrong
       /\ history[s].current.Commits = {}
       /\ ValSet(s) /= {}
       \* everything else is correct
       /\ history[s].current.header /= history[s].verified.header
       /\ history[s].current.header.height > history[s].verified.header.height
       /\ history[s].current.header.time > history[s].verified.header.time
       /\ history[s].current.header.time < history[s].now
       /\ history[s].verified.header.time + TRUSTING_PERIOD > history[s].now

TestLessThanTwoThirdsCommit ==
    /\ \E s \in DOMAIN history :
       \* this is wrong
       LET CMS == history[s].current.Commits
           UVS == history[s].current.header.VS
           TVS == history[s].verified.header.VS
       IN
       /\ history[s].current.header.height > history[s].verified.header.height + 1
       /\ CMS /= {}
       /\ CMS \subseteq UVS
       /\ 3 * Cardinality(CMS) < 2 * Cardinality(UVS)
       /\ 3 * Cardinality(CMS \intersect TVS) < Cardinality(TVS)
       \* everything else is correct
       /\ history[s].current.header /= history[s].verified.header
       /\ history[s].current.header.height > history[s].verified.header.height
       /\ history[s].current.header.time > history[s].verified.header.time
       /\ history[s].current.header.time < history[s].now
       /\ history[s].verified.header.time + TRUSTING_PERIOD > history[s].now
*)
\* Time-related tests

\* Test an execution where a header is received from the future
TestHeaderFromFuture ==
    /\ \E s \in DOMAIN history :
       /\ history[s].now < history[s].current.header.time
       /\ history[s].now < history[s].verified.header.time + TRUSTING_PERIOD

\* Test an execution where the untrusted header time is before the trusted header time
TestUntrustedBeforeTrusted ==
    /\ \E s \in DOMAIN history :
        LET CMS == history[s].current.Commits
            UVS == history[s].current.header.VS
        IN
        /\ history[s].current.header.time < history[s].verified.header.time
        /\ history[s].now < history[s].verified.header.time + TRUSTING_PERIOD
        /\ CMS /= {}
        /\ UVS /= {}
        /\ Cardinality(CMS) < Cardinality(UVS)

\* Test an execution where a header is outside the trusting period
TestHeaderNotWithinTrustingPeriod ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s \in DOMAIN history :
       /\ history[s].now > history[s].verified.header.time + TRUSTING_PERIOD
       /\ history[s].current.header.time < history[s].now

\* Validator set tests

\* Test an execution where the validator sets differ at each step
TestValsetDifferentAllSteps ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \A s1, s2 \in DOMAIN history :
       s1 /= s2  =>
       history[s1].current.header.VS /= history[s2].current.header.VS

TestHalfValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 3
        /\ 2 * Cardinality(ValSet(s1) \intersect ValSet(s2)) < Cardinality(ValSet(s1))

TestHalfValsetChangesVerdictSuccess ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ history[s2].verdict = "SUCCESS"
        /\ Cardinality(ValSet(s1)) >= 3
        /\ 2 * Cardinality(ValSet(s1) \intersect ValSet(s2)) < Cardinality(ValSet(s1))

TestHalfValsetChangesVerdictNotEnoughTrust ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ history[s2].verdict = "NOT_ENOUGH_TRUST"
        /\ Cardinality(ValSet(s1)) >= 3
        /\ 2 * Cardinality(ValSet(s1) \intersect ValSet(s2)) < Cardinality(ValSet(s1))

TestValsetDoubles ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 2
        /\ Cardinality(ValSet(s2)) = 2 * Cardinality(ValSet(s1))

TestValsetHalves ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 4
        /\ Cardinality(ValSet(s1)) = 2 * Cardinality(ValSet(s2))

TestValsetChangesFully ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 2
        /\ ValSet(s1) \intersect ValSet(s2) = {}

TestLessThanThirdValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 4
        /\ ValSet(s2) /= ValSet(s1)
        /\ 3 * Cardinality(ValSet(s2) \ ValSet(s1)) < Cardinality(ValSet(s1))

TestMoreThanTwoThirdsValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 4
        /\ 3 * Cardinality(ValSet(s2) \ ValSet(s1)) > 2 * Cardinality(ValSet(s1))

TestOneThirdValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 3
        /\ 3 * Cardinality(ValSet(s2) \ ValSet(s1)) = Cardinality(ValSet(s1))

TestTwoThirdsValsetChanges ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ Cardinality(ValSet(s1)) >= 3
        /\ 3 * Cardinality(ValSet(s2) \ ValSet(s1)) = 2 * Cardinality(ValSet(s1))

TestLessThanTwoThirdsSign ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s \in DOMAIN history :
       /\ history[s].verified.header.time + TRUSTING_PERIOD > history[s].now
       \* this is wrong
       /\ \E commits \in SUBSET AllNodes:
          \E block \in BC!LightBlocks:
              /\ 3 * Cardinality(commits) < 2 * Cardinality(ValSet(s))
              /\ CopyLightBlockFromChain(block, Height(s))
              /\ LightBlock(s) = [ block EXCEPT !.Commits = commits]

TestMoreThanTwoThirdsSign ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s \in DOMAIN history :
        3 * Cardinality(ValCommits(s)) >= 2 * Cardinality(ValSet(s))


============================================================================

 \* When Apalache is fixed to work with operator params, we should rewrite the validator set tests as shown below

\* A configurable test for two neighbor valsets
TestNeighborValsets(test(_,_)) ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ test(ValSet(s1), ValSet(s2))

\* A configurable test for two neighbor valsets and expected verdict
TestNeighborValsetsVerdict(test(_,_), want_verdict) ==
    /\ Cardinality(DOMAIN fetchedLightBlocks) = TARGET_HEIGHT
    /\ \E s1, s2 \in DOMAIN history :
        /\ s2 = s1 + 1
        /\ test(ValSet(s1), ValSet(s2))
        /\ history[s2].verdict = want_verdict

HalfValsetChanges(vs1, vs2) ==
    /\ Cardinality(vs1) >= 3
    /\ 2 * Cardinality(vs1 \intersect vs2) < Cardinality(vs1)

TestHalfValsetChanges == TestNeighborValsets(HalfValsetChanges)
TestHalfValsetChangesVerdictSuccess == TestNeighborValsetsVerdict(HalfValsetChanges, "SUCCESS")
TestHalfValsetChangesVerdictNotEnoughTrust == TestNeighborValsetsVerdict(HalfValsetChanges, "NOT_ENOUGH_TRUST")

ValsetDoubles(vs1, vs2) ==
    /\ Cardinality(vs1) >= 2
    /\ Cardinality(vs2) = 2 * Cardinality(vs1)

TestValsetDoubles == TestNeighborValsets(ValsetDoubles)
TestValsetDoublesVerdictSuccess == TestNeighborValsetsVerdict(ValsetDoubles, "SUCCESS")
TestValsetDoublesVerdictNotEnoughTrust == TestNeighborValsetsVerdict(ValsetDoubles, "NOT_ENOUGH_TRUST")
