-------------------------- MODULE Lightclient_A_1 ----------------------------
(*
 * A state-machine specification of the lite client, following the English spec:
 * https://github.com/tendermint/spec/tree/master/spec/consensus/light-client
 *
 * Whereas the English specification presents the lite client as a piece of sequential code,
 * which contains non-tail recursion, we specify a state machine that explicitly has
 * a stack of requests in its state. This specification can be easily extended to support
 * multiple requests at the same time, e.g., to reduce latency.
 *) 

EXTENDS Integers, Sequences

\* the parameters of Lite Client
CONSTANTS
  TRUSTED_HEIGHT,
    (* an index of the block header that the light client trusts by social consensus *)
  TO_VERIFY_HEIGHT
    (* an index of the block header that the light client tries to verify *)

VARIABLES       (* see TypeOK below for the variable types *)
  state,        (* the current state of the light client *)
  inEvent,      (* an input event to the light client, e.g., a header from a full node *)
  outEvent,     (* an output event from the light client, e.g., finished with a verdict *)
  requestStack,     (* the stack of requests to be issued by the light client to a full node *)
  storedSignedHeaders     (* the set of headers obtained from a full node*) 

(* the variables of the lite client *)  
lcvars == <<state, outEvent, requestStack, storedSignedHeaders>>  

(* the variables of the client's environment, that is, input events *)
envvars == <<inEvent>>  

(******************* Blockchain instance ***********************************)

\* the parameters that are propagated into Blockchain
CONSTANTS
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  ULTIMATE_HEIGHT,
    (* a maximal height that can be ever reached (modelling artifact) *)
  MAX_POWER
    (* a maximal voting power of a single node *)

\* the state variables of Blockchain, see Blockchain.tla for the details
VARIABLES tooManyFaults, height, minTrustedHeight, blockchain, Faulty

\* All the variables of Blockchain. For some reason, BC!vars does not work
bcvars == <<tooManyFaults, height, minTrustedHeight, blockchain, Faulty>>

(* Create an instance of Blockchain.
   We could write EXTENDS Blockchain, but then all the constants and state variables
   would be hidden inside the Blockchain module.
 *) 
BC == INSTANCE Blockchain_A_1 WITH tooManyFaults <- tooManyFaults, height <- height,
  minTrustedHeight <- minTrustedHeight, blockchain <- blockchain, Faulty <- Faulty

(**************** Environment: User + Full node *************************)
NoEvent == [type |-> "None"]

InEvents ==
        (* start the client given the height to verify (fixed in our modelling) *)
    [type: {"start"}, heightToVerify: BC!Heights]
       \union
        (* receive a signed header that was requested before *)
    [type: {"responseHeader"}, hdr: BC!SignedHeaders]
        \union
    {NoEvent}
    (* most likely, the implementation will have a timeout event, we do not need it here *)

(* initially, the environment is not issuing any requests to the lite client *)
EnvInit ==
    inEvent = NoEvent
    
(* The events that can be generated by the environment (reactor?):
    user requests and node responses *)
EnvNext ==
    (* send a request from the client asking to verify the header at the height heightToVerify *)
    \/ /\ state = "init"
       \* modeling feature, do not start the client before the blockchain is constructed
       /\ height >= TO_VERIFY_HEIGHT
       /\ inEvent' = [type |-> "start", heightToVerify |-> TO_VERIFY_HEIGHT]
    (* send a response from a full node, following the request for header from the light client *)
    \/ /\ state = "working"
       /\ outEvent.type = "requestHeader"
       /\ \E hdr \in BC!SoundSignedHeaders(outEvent.height):
            (* produce an arbitrary header that is either:
               (1) signed by the validators who committed the block, and the block is correct, or
               (2) signed by some faulty validators. *)
            inEvent' = [type |-> "responseHeader", hdr |-> hdr]
            \* if you like to see a counterexample, replace SoundSignedHeaders with SignedHeaders

(************************** Lite client ************************************)

(* the control states of the lite client *) 
States == { "init", "working", "finished" }

(* the events that can be issued by the lite client *)    
OutEvents ==
        (* request the header for a given height from the peer full node *)
    [type: {"requestHeader"}, height: BC!Heights]
        \union
        (* finish the check with a verdict *)
    [type: {"finished"}, verdict: BOOLEAN]  
        \union
    {NoEvent}      

(* Produce a request event for the pivot of the top element of the requestStack *)
RequestHeaderForTopRequest(pStack) ==
    IF pStack = <<>>
    THEN NoEvent    \* the stack is empty, no request
    ELSE
      LET top == Head(pStack) IN
      IF ~top.isLeft
      THEN \* no request for the right branch, the header should have been received
        NoEvent
      ELSE \* request for the left branch, the pivot is the endHeight
        [type |-> "requestHeader", height |-> top.endHeight]

(* When starting the light client *)
OnStart ==
    /\ state = "init"
    /\ inEvent.type = "start"
    /\ inEvent' = NoEvent \* forget the input event
    /\ state' = "working"
        (* the block at trusted height is obtained by the user *)
        \* TODO: the English spec does not make this explicit
    /\ storedSignedHeaders' =
        { [header |-> blockchain[TRUSTED_HEIGHT],
           Commits |-> BC!VS(blockchain[TRUSTED_HEIGHT])] }
        (* The only request on the stack ("left", h1, h2) *)
    /\ IF minTrustedHeight > TRUSTED_HEIGHT
       THEN \* the trusted header is outside of the trusting period
         /\ outEvent' = [type |-> "finished", verdict |-> FALSE]
         /\ UNCHANGED requestStack
       ELSE IF TRUSTED_HEIGHT < TO_VERIFY_HEIGHT
       THEN \* doing upward verification within the trusting period
         LET initStack == << [isLeft |-> TRUE,
                              startHeight |-> TRUSTED_HEIGHT,
                              endHeight |-> inEvent.heightToVerify] >>
         IN
         /\ requestStack' = initStack
         /\ outEvent' = RequestHeaderForTopRequest(initStack)
         \* TODO: update the English spec to address h1 = h2
         \*       (otherwise, the light client would not terminate)
       ELSE
         \* This spec does not implement the downward verification, which only checks hashes
         /\ outEvent' = [type |-> "finished", verdict |-> TRUE]
         /\ UNCHANGED requestStack

(**
 Check whether commits in a signed header are correct with respect to the given
 validator set (DOMAIN votingPower) and votingPower. Additionally, check that
 the header is still within the trusting period.
 *)
Verify(pVotingPower, pSignedHdr) ==
    \* the implementation should check the hashes and other properties of the header
    LET Validators == DOMAIN pVotingPower
        TP == BC!PowerOfSet(pVotingPower, Validators)
        SP == BC!PowerOfSet(pVotingPower, pSignedHdr.Commits \intersect Validators)
    IN
        \* the trusted header is still within the trusting period
    /\ minTrustedHeight <= pSignedHdr.header.height
        \* the commits are signed by the validators
    /\ pSignedHdr.Commits \subseteq BC!VS(pSignedHdr.header)
        \* the 2/3s rule works
    /\ 3 * SP > 2 * TP

(* 
 Check whether we can trust the signedHdr based on trustedHdr
 following the trusting period method.
 This operator is similar to CheckSupport in the English spec.
   
 The parameters have the following meanings:
   - heightToTrust is the height of the trusted header
   - heightToVerify is the height of the header to be verified
   - trustedHdr is the trusted header (not a signed header)
   - signedHdr is the signed header to verify (including commits)
 *)
CheckSupport(pHeightToTrust, pHeightToVerify, pTrustedHdr, pSignedHdr) ==
    IF pHeightToVerify = pHeightToTrust + 1 \* adjacent headers
    THEN pSignedHdr.header.VP = pTrustedHdr.NextVP
    ELSE \* the general case: check 1/3 between the headers  
      LET TP == BC!PowerOfSet(pTrustedHdr.NextVP, BC!NextVS(pTrustedHdr))
          SP == BC!PowerOfSet(pTrustedHdr.NextVP,
                              pSignedHdr.Commits \intersect BC!NextVS(pTrustedHdr))
    IN
    3 * SP > TP

(* Make one step of bisection, roughly one stack frame of Bisection in the English spec *)
OneStepOfBisection(pStoredSignedHeaders) ==
    LET topReq == Head(requestStack)
        lh == topReq.startHeight
        rh == topReq.endHeight
        lhdr == CHOOSE shdr \in pStoredSignedHeaders: shdr.header.height = lh
        rhdr == CHOOSE shdr \in pStoredSignedHeaders: shdr.header.height = rh
    IN
    IF Verify(rhdr.header.VP, rhdr) = FALSE
    THEN [verdict |-> FALSE, newStack |-> <<(* empty *)>> ] \* TERMINATE immediately
    ELSE \* pass only the header lhdr.header and signed header rhdr
      IF CheckSupport(lh, rh, lhdr.header, rhdr)
        (* The header can be trusted, pop the request and return true *)
      THEN [verdict |-> TRUE, newStack |-> Tail(requestStack)]
      ELSE IF lh + 1 = rh \* sequential verification
        THEN (* Sequential verification tells us that the header cannot be trusted. *)
            [verdict |-> FALSE, newStack |-> <<(* empty *)>>] \* TERMINATE immediately
        ELSE (*
             Bisection: schedule search requests for the left and right branches
             (and pop the top element off the stack).
             In contrast to the English spec, these requests are not processed immediately,
             but one-by-one in a depth-first order.
             *)
            LET pivot == (lh + rh) \div 2 \* the pivot point lies in the middle
                rightReq == [isLeft |-> FALSE, startHeight |-> pivot, endHeight |-> rh]
                leftReq ==  [isLeft |-> TRUE, startHeight |-> lh, endHeight |-> pivot]
            IN
            [verdict |-> TRUE, newStack |-> <<leftReq, rightReq>> \o Tail(requestStack)]

(*
 This is where the main loop of bisection is happening.
 The action is activated by a response from a full node.
 The response is sent for a left branch of the bisection.
 *)
OnLeftResponseHeader ==
  /\ state = "working"
  /\ inEvent.type = "responseHeader"
  /\ inEvent' = NoEvent \* forget the input event
  /\ storedSignedHeaders' = storedSignedHeaders \union { inEvent.hdr }  \* save the header
  /\ LET res == OneStepOfBisection(storedSignedHeaders') IN       \* do one step
      /\ requestStack' = res.newStack
      /\ IF res.newStack = << >>        \* end of the bisection
         THEN /\ outEvent' = [type |-> "finished", verdict |-> res.verdict]
              /\ state' = "finished"    \* finish with the given verdict
         ELSE /\ outEvent' = RequestHeaderForTopRequest(res.newStack)
              /\ state' = "working"     \* continue

(*
  This step of bisection is taken when the right branch should be explored.
  It does not need a response from the full node,
  as the pivot header should have been received for the left branch.
 *)
OnRightBranch ==
  /\ state = "working"
  /\ requestStack /= << >>
  /\ LET top == Head(requestStack) IN
    /\ ~top.isLeft          \* the header should have been received already
    /\ inEvent' = NoEvent   \* no input event required
    /\ UNCHANGED storedSignedHeaders \* no headers to store
    /\ LET res == OneStepOfBisection(storedSignedHeaders) IN \* do one step
         /\ requestStack' = res.newStack
         /\ IF res.newStack = << >>        \* end of the bisection
            THEN /\ outEvent' = [type |-> "finished", verdict |-> res.verdict]
                 /\ state' = "finished"    \* finish with the given verdict
            ELSE /\ outEvent' = RequestHeaderForTopRequest(res.newStack)
                 /\ state' = "working"     \* continue

(*
 Initial states of the light client. No requests on the stack, no headers.
 *)
LCInit ==
    /\ state = "init"
    /\ outEvent = NoEvent
    /\ requestStack = <<>>
    /\ storedSignedHeaders = {}

(*
 Actions of the light client: start or do bisection when receiving response.
 *)
LCNext ==
  OnStart \/ OnLeftResponseHeader \/ OnRightBranch
            
            
(********************* Lite client + Environment + Blockchain *******************)
Init ==
    BC!Init /\ EnvInit /\ LCInit

(*
  A system step is made by one of the three components:
    (1) light client, (2) environment (user + full node), (3) blockchain.
 *)
Next ==
    IF outEvent.type = "finished"
         \* the system has stopped, as the lite client has finished
    THEN UNCHANGED <<lcvars, envvars, bcvars>>
    ELSE \* the system is running
        \/ LCNext  /\ UNCHANGED bcvars \* LC resets inEvent *\
        \/ EnvNext /\ UNCHANGED bcvars /\ UNCHANGED lcvars
        \/ BC!Next /\ UNCHANGED lcvars /\ UNCHANGED envvars

(************************* Types ******************************************)

TypeOK ==
    /\ state \in States
    /\ inEvent \in InEvents
    /\ outEvent \in OutEvents
    /\ requestStack \in Seq([isLeft: BOOLEAN, startHeight: BC!Heights, endHeight: BC!Heights])
    /\ storedSignedHeaders \subseteq BC!SignedHeaders

(************************* Properties ******************************************)

(* The properties to check *)
\* check this property to get an example of a terminating light client
NeverStart == state /= "working"

NeverFinishNegative ==
  ~(outEvent.type = "finished" /\ outEvent.verdict = FALSE)

NeverFinishPositive ==
  ~(outEvent.type = "finished" /\ outEvent.verdict = TRUE)

NeverFinishPositiveWithFaults ==
  ~(outEvent.type = "finished" /\ outEvent.verdict = TRUE /\ tooManyFaults)

\* Correctness states that all the obtained headers are exactly like in the blockchain.
\* This formula is equivalent to Accuracy in the English spec, that is, A => B iff ~B => ~A.
\*
\* Lite Client Accuracy: If header h was not generated by an instance of Tendermint consensus,
\* then the lite client should never set trust(h) to true.
CorrectnessInv ==
    (outEvent.type = "finished" /\ outEvent.verdict = TRUE)
        => (\A shdr \in storedSignedHeaders: shdr.header = blockchain[shdr.header.height])

\* CorrectnessInv holds only under the assumption of the Tendermint security model.
\* Hence, Correctness is restricted to the case when there are less than 1/3 of faulty validators.
Correctness ==
    []~tooManyFaults => CorrectnessInv

\* There are no two headers of the same height
NoDupsInv ==
    \A shdr1, shdr2 \in storedSignedHeaders:
      (shdr1.header.height = shdr2.header.height) => (shdr1 = shdr2)

\* Check that the sequence of the headers in storedSignedHeaders satisfies checkSupport pairwise
\* This property is easily violated, whenever a header cannot be trusted anymore.
StoredHeadersAreSound ==
    outEvent.type = "finished" /\ outEvent.verdict = TRUE
        =>
        \A left, right \in storedSignedHeaders: \* for every pair of different stored headers
            \/ left.header.height >= right.header.height
               \* either there is a header between them
            \/ \E middle \in storedSignedHeaders:
                /\ left.header.height < middle.header.height
                /\ middle.header.height < right.header.height
               \* or we can verify the right one using the left one
            \/ CheckSupport(left.header.height, right.header.height, left.header, right)

\* An improved version of StoredHeadersAreSound, assuming that a header may be not trusted
StoredHeadersAreSoundOrNotTrusted ==
    outEvent.type = "finished" /\ outEvent.verdict = TRUE
        =>
        \A left, right \in storedSignedHeaders: \* for every pair of different stored headers
            \/ left.header.height >= right.header.height
               \* either there is a header between them
            \/ \E middle \in storedSignedHeaders:
                /\ left.header.height < middle.header.height
                /\ middle.header.height < right.header.height
               \* or we can verify the right one using the left one
            \/ CheckSupport(left.header.height, right.header.height, left.header, right)
               \* or the left header is outside the trusting period, so no guarantees 
            \/ minTrustedHeight > left.header.height

\* The lite client must always terminate under the given pre-conditions.
\* E.g., assuming that the full node always replies.
TerminationPre ==
       \* the user and the full node take steps, if they can
    /\ WF_envvars(EnvNext)
       \* and the lite client takes steps, if it can
    /\ WF_lcvars(LCNext)
       \* eventually, the blockchain is sufficiently high, and
       \* the blockchain is never dead, see NeverStuck in Blockchain.tla
    /\ <>[](height >= minTrustedHeight /\ height >= TO_VERIFY_HEIGHT)

\* Given the preconditions, the lite client eventually terminates.        
Termination ==
  TerminationPre => <>(outEvent.type = "finished")        

\* This property states that whenever the light client finishes with a positive outcome,
\* the trusted header is still within the trusting period.
\* The current spec most likely violates this property.
PositiveBeforeTrustedHeaderExpires ==
    (outEvent.type = "finished" /\ outEvent.verdict = TRUE)
        => (minTrustedHeight <= TRUSTED_HEIGHT)

\* Lite Client Completeness: If header h was correctly generated by an instance
\* of Tendermint consensus (and its age is less than the trusting period),
\* then the lite client should eventually set trust(h) to true.
\*
\* Note that Completeness assumes that the lite client communicates with a correct full node.
\*
\* We decompose completeness into Termination (liveness) and Precision (safety).
\* Once again, Precision is an inverse version of the safety property in Completeness,
\* as A => B is logically equivalent to ~B => ~A. 
PrecisionInv ==
    (outEvent.type = "finished" /\ outEvent.verdict = FALSE)
      => \/ minTrustedHeight > TRUSTED_HEIGHT \* outside of the trusting period
         \/ \E shdr \in storedSignedHeaders:
                 \* the full node lied to the lite client about the block header
              \/ shdr.header /= blockchain[shdr.header.height]
                 \* the full node lied to the lite client about the commits
              \/ shdr.Commits /= BC!VS(shdr.header)

\* the old invariant that was found to be buggy by TLC
PrecisionBuggyInv ==
    (outEvent.type = "finished" /\ outEvent.verdict = FALSE)
      => \/ minTrustedHeight > TRUSTED_HEIGHT \* outside of the trusting period
         \/ \E shdr \in storedSignedHeaders:
              \* the full node lied to the lite client about the block header
              shdr.header /= blockchain[shdr.header.height]

\* Precision states that PrecisionInv always holds true.
\* When we independently check Termination and PrecisionInv,
\* there is no need to check Completeness.         
Completeness ==
    Termination /\ []PrecisionInv

(************************** MODEL CHECKING ************************************)
(*
  # Experiment 1 (2 validators + 1 full node, we found many logical bugs with that).
  Run TLC with the following parameters (the figures are given for 3 CPU cores):
  
  ULTIMATE_HEIGHT <- 3,
  MAX_POWER <- 1,
  TO_VERIFY_HEIGHT <- 3,
  TRUSTED_HEIGHT <- 1,
  AllNodes <- { A_p1, A_p2 } \* choose symmetry reduction for model values
  
   * Termination: satisfied after 13 min.
   * PrecisionInv: satisfied after 42 sec.
   * Correctness: satisfied after 45 sec.
   * CorrectnessInv: violation after 7 sec: lastCommit may deviate
   * NoDupsInv: satisfied after 39 sec.
   * StoredHeadersAreSound: violation after 9 sec.
   * StoredHeadersAreSoundOrNonTrusted: correct after 13 min.
   * PositiveBeforeTrustedHeaderExpires: violation after 10 sec.
   * PrecisionBuggyInv: violation after 3 sec.

  # Experiment 2 (the example that makes sense: 4 validators + 1 full node).
  Run TLC with the following parameters:
  
  ULTIMATE_HEIGHT <- 3,
  MAX_POWER <- 1,
  TO_VERIFY_HEIGHT <- 3,
  TRUSTED_HEIGHT <- 1,
  AllNodes <- { A_p1, A_p2, A_p3, A_p4 } \* choose symmetry reduction for model values
  
   * Deadlocks: a deadlock occurs when minTrustedHeight > height.
   * PrecisionInv: ???
   * CorrectnessInv: ???
   * Termination: ??? 
   * StoredHeadersAreSoundOrNonTrusted: ???
 *)

=============================================================================
\* Modification History
\* Last modified Sat Nov 30 19:55:46 CET 2019 by igor
\* Created Wed Oct 02 16:39:42 CEST 2019 by igor
