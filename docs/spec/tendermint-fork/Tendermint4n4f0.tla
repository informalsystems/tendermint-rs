----------------------------- MODULE Tendermint4n4f0 --------------------
(*
 A TLA+ specification of Tendermint consensus by Ethan Buchman, Jae Kwon, and Zarko Milosevic.
 
 For the moment, we assume the following:
 
   1. Every process has the voting power of 1.
   2. Timeouts are non-deterministic (works for safety).
   3. The proposer function is non-deterministic (works for safety).
 
 Encoded in TLA+ by Igor Konnov. It took me 4 hours to translate the pseudo-code to TLA+.

 Version 3: assuming that f = 1,
    using AtLeastT1(S) and AtLeastT2(s) instead of
    Cardinality(S) >= f + 1 and Cardinality(S) >= 2f + 1
 *)
 
EXTENDS Integers, FiniteSets

CONSTANTS PropFun     \* the proposer function

N == 4 \* the total number of processes: correct and faulty
T == 0 \* an upper bound on the number of Byzantine processes
F == 0 \* the number of Byzantine processes
Procs == 1..N-F
Faulty == N-F+1..N
Heights == 0..1 \* the set of consensus instances
Rounds == 0..2  \* the set of possible rounds, give a bit more freedom to the solver
ValidValues == {0, 1}     \* e.g., picked by a correct process, or a faulty one
InvalidValues == {2}    \* e.g., sent by a Byzantine process
Values == ValidValues \cup InvalidValues \* all values
nil == -1

\* these are two tresholds that are used in the algorithm
THRESHOLD1 == T + 1
THRESHOLD2 == 2 * T + 1 

(* APALACHE-BEGIN annotations *)
a <: b == a

MT == [type |-> STRING, src |-> Int, h |-> Int, round |-> Int,
       proposal |-> Int, validRound |-> Int, hash |-> Int]
ProposalT == [src |-> Int, proposal |-> Int, validRound |-> Int]
PrevoteT == [src |-> Int, hash |-> Int]
PrecommitT == PrevoteT
ET == [type |-> STRING, src |-> Int, h |-> Int, round |-> Int, value |-> Int] \* processed events, "for the first time"

\* this is an optimization, in order to avoid cardinality constraints
\* one faulty process:
\*AtLeastT1(S) == \E x, y \in S: x /= y
\*AtLeastT2(S) == \E x, y, z \in S: x /= y /\ x /= z /\ y /= z
\* no faults:
AtLeastT1(S) == \E x \in S: TRUE
AtLeastT2(S) == \E x, y \in S: x /= y

ValueT == Int
RoundT == Int
TimeoutT == <<Int, Int, Int>> \* process, height, round 
(* APALACHE-END *)

FaultyProposals ==
    [src: Faulty, proposal: Values, validRound: Rounds \cup {-1}]

FaultyPrevotes == [src: Faulty, hash: Values]

FaultyPrecommits == [src: Faulty, hash: Values]

ConstInit == PropFun \in [Heights \X Rounds -> Procs]


\* these variables are exactly as in the pseudo-code
VARIABLES h, round, step, decision, lockedValue, lockedRound, validValue, validRound 

\* book-keeping variables
VARIABLES msgsPropose, \* the propose messages broadcasted in the system, a function Heights \X Rounds -> set of messages
          msgsPrevote, \* the prevote messages broadcasted in the system, a function Heights \X Rounds -> set of messages  
          msgsPrecommit, \* the precommit messages broadcasted in the system, a function Heights \X Rounds -> set of messages  
          oldEvents,  \* the messages processed once, as expressed by "for the first time"
          timeoutPropose, \* a set of proposed timeouts: <<process, height, round>>
          timeoutPrevote, \* a set of proposed timeouts: <<process, height, round>>
          timeoutPrecommit \* a set of proposed timeouts: <<process, height, round>>

\* this is needed for UNCHANGED
vars == <<h, round, step, decision, lockedValue, lockedRound, validValue,
          validRound, msgsPropose, msgsPrevote, msgsPrecommit, oldEvents, timeoutPropose, timeoutPrevote, timeoutPrecommit>>

\* A function which gives the proposer for a given round at a given height.
\* Here we use round robin. As Procs and Faulty are assigned non-deterministically,
\* it does not really matter who starts first.
Proposer(ht, rd) == PropFun[ht, rd] \*1 + ((ht + rd) % N)

Id(v) == v

IsValid(v) == v \in ValidValues

\* here we start with StartRound(0)
Init ==
    /\ h = [p \in Procs |-> 0]
    /\ round = [p \in Procs |-> 0]
    /\ step = [p \in Procs |-> "PROPOSE"]
    /\ decision = [p \in Procs |-> [ht \in Heights |-> nil]]
    /\ lockedValue = [p \in Procs |-> nil]
    /\ lockedRound = [p \in Procs |-> -1]
    /\ validValue = [p \in Procs |-> nil]
    /\ validRound = [p \in Procs |-> -1]
    /\ \E v \in Values:
        /\ msgsPropose =
            [<<ht, rd>> \in Heights \X Rounds |->
                IF ht = 0 /\ rd = 0
                THEN {[src |-> Proposer(0, 0),
                        proposal |-> v, validRound |-> -1]} \cup FaultyProposals
                ELSE FaultyProposals \* only faulty messages
            ]
    /\ msgsPrevote = [<<ht, rd>> \in Heights \X Rounds |-> FaultyPrevotes]
    /\ msgsPrecommit = [<<ht, rd>> \in Heights \X Rounds |-> FaultyPrecommits]
    /\ oldEvents = {} <: {ET}
    /\ timeoutPropose = { <<p, 0, 0>> : p \in Procs \ {Proposer(0, 0)}} 
    /\ timeoutPrevote = {} <: {TimeoutT}   \* no PREVOTE timeouts
    /\ timeoutPrecommit = {} <: {TimeoutT} \* no PRECOMMIT timeouts

\* lines 22-27        
UponProposalInPropose(p) ==
    \E v \in Values:
      /\ step[p] = "PROPOSE" \* line 22
      /\ [src |-> Proposer(h[p], round[p]), proposal |-> v, validRound |-> -1]
            \in msgsPropose[h[p], round[p]] \* line 22
      /\ LET isGood == IsValid(v) /\ (lockedRound[p] = -1 \/ lockedValue[p] = v) IN \* line 23
         LET newMsgs == {[src |-> p, hash |-> IF isGood THEN Id(v) ELSE nil]}
         IN  \* lines 24-26
         msgsPrevote' = [msgsPrevote EXCEPT ![h[p], round[p]] =
                            msgsPrevote[h[p], round[p]] \cup newMsgs] 
      /\ step' = [step EXCEPT ![p] = "PREVOTE"]
      /\ UNCHANGED <<h, round, decision, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrecommit, oldEvents,
                     timeoutPropose, timeoutPrevote, timeoutPrecommit>>

\* lines 28-33        
UponProposalInProposeAndPrevote(p) ==
    \E v \in Values, vr \in Rounds:
      /\ step[p] = "PROPOSE" /\ 0 <= vr /\ vr < round[p] \* line 28, the while part
      /\ [src |-> Proposer(h[p], round[p]), proposal |-> v, validRound |-> vr]
            \in msgsPropose[h[p], round[p]] \* line 28
      /\ LET PV == { m \in msgsPrevote[h[p], vr]: m.hash = Id(v) } IN
         AtLeastT2(PV) \* line 28
      /\ LET isGood == IsValid(v) /\ (lockedRound[p] <= vr \/ lockedValue[p] = v) IN \* line 29
         LET newMsgs == {[src |-> p, hash |-> IF isGood THEN Id(v) ELSE nil]}
         IN \* lines 30-32
         msgsPrevote' = [msgsPrevote EXCEPT ![h[p], round[p]] =
                            msgsPrevote[h[p], round[p]] \cup newMsgs] 
      /\ step' = [step EXCEPT ![p] = "PREVOTE"]
      /\ UNCHANGED <<h, round, decision, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrecommit, oldEvents,
                     timeoutPropose, timeoutPrevote, timeoutPrecommit>>

\* lines 34-35        
UponPrevoteFirstTime(p) ==
      /\ step[p] = "PREVOTE" \* line 34
      /\ AtLeastT2(msgsPrevote[h[p], round[p]]) \* line 34
      /\  LET event == [type |-> "PREVOTE", src |-> p,
                       h |-> h[p], round |-> round[p], value |-> nil]
          IN
        /\ event \notin oldEvents               \* for the first time
        /\ oldEvents' = oldEvents \cup {event}  \* process it only once
      /\ timeoutPrevote' = timeoutPrevote \cup {<<p, h[p], round[p]>>} \* line 35 
      /\ UNCHANGED <<h, round, step, decision, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                     timeoutPropose, timeoutPrecommit>>

\* lines 36-46        
UponProposalInPrevoteOrCommitAndPrevote(p) ==
    \E v \in ValidValues, vr \in Rounds \cup {-1}:
      /\ step[p] \in {"PREVOTE", "PRECOMMIT"} \* line 36
      /\ [src |-> Proposer(h[p], round[p]), proposal |-> v, validRound |-> vr]
            \in msgsPropose[h[p], round[p]] \* line 36
      /\ LET event == [type |-> "PREVOTE", src |-> p, h |-> h[p],
                       round |-> round[p], value |-> Id(v)] IN
          /\ event \notin oldEvents               \* for the first time
          /\ oldEvents' = oldEvents \cup {event}  \* record that it should not happen again
      /\ LET PV == { m \in msgsPrevote[h[p], round[p]]: m.hash = Id(v) } IN
         AtLeastT2(PV) \* line 36
      /\ lockedValue' =
         IF step[p] = "PREVOTE"
         THEN [lockedValue EXCEPT ![p] = v] \* line 38
         ELSE lockedValue \* else of line 37
      /\ lockedRound' =     
         IF step[p] = "PREVOTE"
         THEN [lockedRound EXCEPT ![p] = round[p]] \* line 39
         ELSE lockedRound \* else of line 37
      /\ LET newMsgs ==
           IF step[p] = "PREVOTE"
           THEN {[src |-> p, hash |-> Id(v)]} \* line 40
           ELSE {} <: {PrecommitT}
         IN \* else of line 37
         msgsPrecommit' = [msgsPrecommit EXCEPT ![h[p], round[p]] =
                            msgsPrecommit[h[p], round[p]] \cup newMsgs] \* line 40, or else of 37 
      /\ step' = 
        IF step[p] = "PREVOTE"
        THEN [step EXCEPT ![p] = "PRECOMMIT"]
        ELSE step \* line 41
      /\ validValue' = [validValue EXCEPT ![p] = v] \* line 42
      /\ validRound' = [validRound EXCEPT ![p] = round[p]] \* line 43
      /\ UNCHANGED <<h, round, decision, msgsPropose, msgsPrevote, timeoutPropose, timeoutPrevote, timeoutPrecommit>>

\* Apparently, this action is needed to deal with a value proposed by a Byzantine process
\* lines 44-46        
UponPrevoteNil(p) ==
      /\ step[p] = "PREVOTE" \* line 44
      /\ LET PV == { m \in msgsPrevote[h[p], round[p]]: m.hash = nil }
         IN
         AtLeastT2(PV) \* line 34
      /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
      /\ LET newMsgs == {[src |-> p, hash |-> nil]} \* line 45
         IN
         msgsPrecommit' = [msgsPrecommit EXCEPT ![h[p], round[p]] =
                            msgsPrecommit[h[p], round[p]] \cup newMsgs] \* line 45 
      /\ UNCHANGED <<h, round, decision, lockedValue, lockedRound, validValue,
                     validRound, oldEvents, msgsPropose, msgsPrevote, timeoutPropose, timeoutPrevote, timeoutPrecommit>>

\* lines 47-48        
UponPrecommitFirstTime(p) ==
      /\ AtLeastT2(msgsPrecommit[h[p], round[p]]) \* line 47
      /\ LET event == [type |-> "PRECOMMIT", src |-> p,
                       h |-> h[p], round |-> round[p], value |-> nil]
          IN
        /\ event \notin oldEvents               \* for the first time
        /\ oldEvents' = oldEvents \cup {event}  \* process it only once
      /\ timeoutPrecommit' = timeoutPrecommit \cup {<<p, h[p], round[p]>>} \* line 48 
      /\ UNCHANGED <<h, round, step, decision, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                     timeoutPropose, timeoutPrevote>>

\* lines 11-21
StartRound(p, ht, r) ==
   /\ round' = [round EXCEPT ![p] = r]
   /\ step' = [step EXCEPT ![p] = "PROPOSE"]
   /\ \E v \in ValidValues: \* lines 14-21
     LET proposal == IF validValue[p] /= nil THEN validValue[p] ELSE v IN
     LET newMsgs ==
        IF p = Proposer(ht, r)
        THEN {[src |-> p, proposal |-> proposal, validRound |-> validRound[p]]}
        ELSE {} <: {ProposalT}
     IN
     msgsPropose' = [msgsPropose EXCEPT ![ht, r] =
                            msgsPropose[ht, r] \cup newMsgs] \* line 19 
   /\  LET newTimeouts == \* line 21
         IF p = Proposer(ht, r)
         THEN {} <: {TimeoutT} \* no new timeouts
         ELSE { <<p, ht, r>> }
       IN 
       timeoutPropose' = timeoutPropose \cup newTimeouts

\* lines 49-54        
UponProposalInPrecommitNoDecision(p) ==
    /\ h[p] + 1 \in Heights
         \* THIS IS NOT PART OF THE ORIGINAL ALGORITHM, BUT A SAFEGUARD TO PREVENT ROUNDS FROM OVERFLOWING
    /\ decision[p][h[p]] = nil \* line 49
    /\ \E v \in ValidValues (* line 50*) , r \in Rounds, vr \in Rounds \cup {-1}:
      /\ [src |-> Proposer(h[p], r), proposal |-> v, validRound |-> vr]
            \in msgsPropose[h[p], r] \* line 49
      /\ LET PV == { m \in msgsPrecommit[h[p], r]: m.hash = Id(v) } IN
         AtLeastT2(PV) \* line 49
      /\ decision' = [decision EXCEPT ![p][h[p]] = v] \* update the decision, line 51
      /\ h' = [h EXCEPT ![p] = h[p] + 1] \* line 52
        \* line 53
      /\ lockedRound' = [lockedRound EXCEPT ![p] = -1]
      /\ lockedValue' = [lockedValue EXCEPT ![p] = nil]
      /\ validRound' = [validRound EXCEPT ![p] = -1]
      /\ validValue' = [validValue EXCEPT ![p] = nil]
        \* What does it mean to reset the message buffer? Do it for one process only?
      /\ StartRound(p, h[p] + 1, 0)
      /\ UNCHANGED <<oldEvents, msgsPrevote, msgsPrecommit, timeoutPrevote, timeoutPrecommit>>

\* lines 55-56
UponCatchupRound(p) ==
    \E r \in Rounds:
        /\ r > round[p]
        /\ LET ps == { m.src : m \in msgsPropose[h[p], r]} IN
           LET pv == { m.src : m \in msgsPrevote[h[p], r]} IN
           LET pc == { m.src : m \in msgsPrecommit[h[p], r]} IN
            AtLeastT1(ps \cup pv \cup pc) \* line 55
        /\ StartRound(p, h[p], r)
        /\ UNCHANGED <<h, decision, lockedValue, lockedRound, validValue,
                       validRound, oldEvents, msgsPrevote, msgsPrecommit,
                       timeoutPrevote, timeoutPrecommit>>

\* lines 57-60
OnTimeoutPropose(p) ==
    \E tm \in timeoutPropose: \* a timeout occurs
        /\ tm[1] = p
        /\ timeoutPropose' = timeoutPropose \ {tm} \* remove from the future timeouts
        /\ LET UpdateNeeded == tm[2] = h[p] /\ tm[3] = round[p] /\ step[p] = "PROPOSE" IN
            /\ step' = IF UpdateNeeded THEN [step EXCEPT ![p] = "PREVOTE"] ELSE step \* line 60
            /\ LET newMsgs ==
               IF UpdateNeeded
               THEN {[src |-> tm[1], hash |-> nil]} \* line 59
               ELSE {} <: {PrevoteT} \* else of line 58
               IN  \* line 59, or else of 58
               msgsPrevote' = [msgsPrevote EXCEPT ![tm[2], tm[3]] =
                                    msgsPrevote[tm[2], tm[3]] \cup newMsgs] 
        /\ UNCHANGED <<h, round, decision, lockedValue, lockedRound, validValue,
                       validRound, oldEvents, msgsPropose, msgsPrecommit,
                       timeoutPrevote, timeoutPrecommit>>

\* lines 61-64
OnTimeoutPrevote(p) ==
    \E tm \in timeoutPrevote: \* a timeout occurs
        /\ tm[1] = p
        /\ timeoutPrevote' = timeoutPrevote \ {tm} \* remove from the future timeouts
        /\ LET UpdateNeeded == tm[2] = h[p] /\ tm[3] = round[p] /\ step[p] = "PREVOTE" IN
            /\ step' =
                IF UpdateNeeded
                THEN [step EXCEPT ![p] = "PRECOMMIT"]
                ELSE step \* line 64
            /\ LET newMsgs ==
                IF UpdateNeeded
                THEN {[src |-> tm[1], hash |-> nil]} \* line 63
                ELSE {} <: {PrecommitT} \* else of line 62
               IN \* line 63, or else of 62
               msgsPrecommit' = [msgsPrecommit EXCEPT ![tm[2], tm[3]] =
                                    msgsPrecommit[tm[2], tm[3]] \cup newMsgs] 
        /\ UNCHANGED <<h, round, decision, lockedValue, lockedRound, validValue,
                       validRound, oldEvents, msgsPropose, msgsPrevote,
                       timeoutPropose, timeoutPrecommit>>

\* lines 65-67
OnTimeoutPrecommitOutside(p) ==
    \E tm \in timeoutPrecommit: \* a timeout occurs
        /\ tm[1] = p
        /\ (tm[2] /= h[p] \/ tm[3] /= round[p]) \* but we are in another round or height
        /\ timeoutPrecommit' = timeoutPrecommit \ {tm} \* remove from the future timeouts
        /\ UNCHANGED <<h, round, step, decision, lockedValue, lockedRound, validValue,
                       validRound, msgsPropose, msgsPrevote, msgsPrecommit, oldEvents,
                       timeoutPropose, timeoutPrevote>>

\* lines 65-67
OnTimeoutPrecommitInside(p) ==
    /\ round[p] + 1 \in Rounds
       \* THIS IS NOT PART OF THE ORIGINAL ALGORITHM, BUT A SAFEGUARD TO PREVENT ROUNDS FROM OVERFLOWING
    /\ <<p, h[p], round[p]>> \in timeoutPrecommit \* the timeout occurs for the current round and height
    /\ timeoutPrecommit' = timeoutPrecommit \ {<<p, h[p], round[p]>>} \* remove from the future timeouts
    /\ StartRound(p, h[p], round[p] + 1)
    /\ UNCHANGED <<h, decision, lockedValue, lockedRound, validValue,
                   validRound, oldEvents, msgsPrevote, msgsPrecommit, timeoutPrevote>>
            

Next ==
    \/ \E p \in Procs:
        \/ UponProposalInPropose(p)
        \/ UponProposalInProposeAndPrevote(p)
        \/ UponPrevoteFirstTime(p)
        \/ UponProposalInPrevoteOrCommitAndPrevote(p)
        \/ UponPrevoteNil(p) \* FIXME: disabled for model checking
        \/ UponPrecommitFirstTime(p)
        \/ UponProposalInPrecommitNoDecision(p)
        \/ UponCatchupRound(p)
        \/ OnTimeoutPropose(p)
        \/ OnTimeoutPrevote(p)
        \/ OnTimeoutPrecommitOutside(p)
        \/ OnTimeoutPrecommitInside(p)
    \* a safeguard to prevent deadlocks when the algorithm goes to further heights or rounds
    \*\/ UNCHANGED vars

\* safety
Agreement ==
    \A p, q \in Procs, ht \in Heights:
        \/ decision[p][ht] = nil
        \/ decision[q][ht] = nil
        \/ decision[p][ht] = decision[q][ht]
        
\* simple reachability properties to make sure that the algorithm is doing anything useful
NoDecision == \A p \in Procs, ht \in Heights: decision[p][ht] = nil

NoPrecommit == \A p \in Procs: step[p] /= "PRECOMMIT"       
      
NoTwoLockedValues ==
  \A p, q \in Procs:
      h[p] = h[q] => lockedValue[p] = nil \/ lockedValue[q] = nil \/ lockedValue[p] = lockedValue[q] 
      
NoTwoLockedRounds ==
  \A p, q \in Procs:
      h[p] = h[q] => lockedRound[p] = -1 \/ lockedRound[q] = -1 \/ lockedRound[p] = lockedRound[q] 


=============================================================================
\* Modification History
\* Last modified Fri Mar 22 08:57:10 CET 2019 by igor
\* Created Fri Mar 15 10:30:17 CET 2019 by igor
