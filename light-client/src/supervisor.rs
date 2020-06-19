use crate::{
    bail,
    callback::Callback,
    components::io::IoError,
    errors::{Error, ErrorKind},
    evidence::EvidenceReporter,
    fork_detector::{Fork, ForkDetection, ForkDetector},
    light_client::LightClient,
    peer_list::PeerList,
    state::State,
    store::VerifiedStatus,
    types::{Height, LightBlock, PeerId},
};

use tendermint::evidence::{ConflictingHeadersEvidence, Evidence};

use contracts::pre;
use crossbeam_channel as channel;

/// Type alias for readability
pub type VerificationResult = Result<LightBlock, Error>;

/// Events which are exchanged between the `Supervisor` and its `Handle`s.
#[derive(Debug)]
pub enum Event {
    // Inputs
    /// Terminate the supervisor process
    Terminate(Callback<()>),
    /// Verify to the highest height, call the provided callback with result
    VerifyToHighest(Callback<VerificationResult>),
    /// Verify to the given height, call the provided callback with result
    VerifyToTarget(Height, Callback<VerificationResult>),

    // Outputs
    /// The supervisor has terminated
    Terminated,
    /// The verification has succeded
    VerificationSuccess(Box<LightBlock>),
    /// The verification has failed
    VerificationFailure(Error),
}

/// An light client `Instance` packages a `LightClient` together with its `State`.
#[derive(Debug)]
pub struct Instance {
    /// The light client for this instance
    pub light_client: LightClient,
    /// The state of the light client for this instance
    pub state: State,
}

impl Instance {
    /// Constructs a new instance from the given light client and its state.
    pub fn new(light_client: LightClient, state: State) -> Self {
        Self {
            light_client,
            state,
        }
    }
}

/// The supervisor manages multiple light client instances, of which one
/// is deemed to be the primary instance through which blocks are retrieved
/// and verified. The other instances are considered as witnesses
/// which are consulted to perform fork detection.
///
/// If primary verification fails, the primary client is removed and a witness
/// is promoted to primary. If a witness is deemed faulty, then the witness is
/// removed.
///
/// The supervisor is intended to be ran in its own thread, and queried
/// via a `Handle`, sync- or asynchronously.
///
/// ## Example
///
/// ```rust,ignore
/// let mut supervisor: Supervisor = todo!();
/// let mut handle = supervisor.handle();
///
/// // Spawn the supervisor in its own thread.
/// std::thread::spawn(|| supervisor.run());
///
/// loop {
///     // Asynchronously query the supervisor via a handle
///     handle.verify_to_highest_async(|result| match result {
///         Ok(light_block) => {
///             println!("[ info  ] synced to block {}", light_block.height());
///         }
///         Err(e) => {
///             println!("[ error ] sync failed: {}", e);
///         }
///     });
///
///     std::thread::sleep(Duration::from_millis(800));
/// }
/// ```
///
/// ## TODO
/// - Report evidence for forks
pub struct Supervisor {
    /// List of peers (primary + witnesses)
    peers: PeerList,
    /// An instance of the fork detector
    fork_detector: Box<dyn ForkDetector>,
    /// Reporter of fork evidence
    evidence_reporter: Box<dyn EvidenceReporter>,
    /// Channel through which to reply to `Handle`s
    sender: channel::Sender<Event>,
    /// Channel through which to receive events from the `Handle`s
    receiver: channel::Receiver<Event>,
}

impl std::fmt::Debug for Supervisor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Supervisor")
            .field("peers", &self.peers)
            .finish()
    }
}

// Ensure the `Supervisor` can be sent across thread boundaries.
static_assertions::assert_impl_all!(Supervisor: Send);

impl Supervisor {
    /// Constructs a new supevisor from the given list of peers and fork detector instance.
    pub fn new(
        peers: PeerList,
        fork_detector: impl ForkDetector + 'static,
        evidence_reporter: impl EvidenceReporter + 'static,
    ) -> Self {
        let (sender, receiver) = channel::unbounded::<Event>();

        Self {
            peers,
            sender,
            receiver,
            fork_detector: Box::new(fork_detector),
            evidence_reporter: Box::new(evidence_reporter),
        }
    }

    /// Verify to the highest block.
    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_highest(&mut self) -> VerificationResult {
        self.verify(None)
    }

    /// Verify to the block at the given height.
    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        self.verify(Some(height))
    }

    /// Verify either to the latest block (if `height == None`) or to a given block (if `height == Some(height)`).
    #[pre(self.peers.primary().is_some())]
    fn verify(&mut self, height: Option<Height>) -> VerificationResult {
        // While there is a primary peer left:
        while let Some(primary) = self.peers.primary_mut() {
            // Perform light client core verification for the given height (or highest).
            let verdict = match height {
                None => primary.light_client.verify_to_highest(&mut primary.state),
                Some(height) => primary
                    .light_client
                    .verify_to_target(height, &mut primary.state),
            };

            match verdict {
                // Verification succeeded, let's peform fork detection
                Ok(light_block) => {
                    // SAFETY: There must be a latest trusted state otherwise verification would have failed.
                    let trusted_state = primary
                        .state
                        .light_store
                        .highest(VerifiedStatus::Verified)
                        .unwrap();

                    // Perform fork detection with the highest verified block as the trusted state.
                    let outcome = self.detect_forks(&light_block, &trusted_state)?;

                    match outcome {
                        // There was a fork or a faulty peer
                        Some(forks) => {
                            let mut forked = Vec::with_capacity(forks.len());

                            for fork in forks {
                                match fork {
                                    // An actual fork was detected, report evidence and record forked peer.
                                    Fork::Forked { primary, witness } => {
                                        let provider = witness.provider;
                                        self.report_evidence(provider, &primary, &witness)?;

                                        forked.push(provider);
                                    }
                                    // A witness has been deemed faulty, remove it from the peer list.
                                    Fork::Faulty(block, _error) => {
                                        self.peers.mark_witness_as_faulty(block.provider);
                                        // TODO: Log/record the error
                                    }
                                }
                            }

                            if !forked.is_empty() {
                                // Fork detected, exiting
                                bail!(ErrorKind::ForkDetected(forked))
                            }
                        }
                        None => {
                            // No fork detected, exiting
                            // TODO: Send to relayer, maybe the run method does this?
                            return Ok(light_block);
                        }
                    }
                }
                // Verification failed
                Err(_err) => {
                    // Swap primary, and continue with new primary, if there is any witness left.
                    self.peers.swap_primary()?;
                    // TODO: Log/record error
                    continue;
                }
            }
        }

        bail!(ErrorKind::NoWitnessLeft)
    }

    /// Report the given evidence of a fork.
    fn report_evidence(
        &mut self,
        provider: PeerId,
        primary: &LightBlock,
        witness: &LightBlock,
    ) -> Result<(), Error> {
        let evidence = ConflictingHeadersEvidence::new(
            primary.signed_header.clone(),
            witness.signed_header.clone(),
        );

        self.evidence_reporter
            .report(Evidence::ConflictingHeaders(Box::new(evidence)), provider)
            .map_err(ErrorKind::Io)?;

        Ok(())
    }

    /// Perform fork detection with the given block and trusted state.
    #[pre(self.peers.primary().is_some())]
    fn detect_forks(
        &mut self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
    ) -> Result<Option<Vec<Fork>>, Error> {
        if self.peers.witnesses().is_empty() {
            bail!(ErrorKind::NoWitnesses);
        }

        let result =
            self.fork_detector
                .detect_forks(light_block, &trusted_state, self.peers.witnesses());

        match result {
            Ok(ForkDetection::Detected(forks)) => Ok(Some(forks)),
            Ok(ForkDetection::NotDetected) => Ok(None),
            Err(e) => match e.kind() {
                // TODO: Clean this up
                // Some RPC request timed out, this peer might be down so let's
                // remove it from the witnesses, and bubble the error up.
                ErrorKind::Io(IoError::Timeout(peer)) => {
                    self.peers.mark_witness_as_faulty(*peer);
                    Err(e)
                }
                _ => Err(e),
            },
        }
    }

    /// Create a new handle to this supervisor.
    pub fn handle(&mut self) -> Handle {
        Handle::new(self.sender.clone())
    }

    /// Run the supervisor event loop in the same thread.
    ///
    /// This method should typically be called within a new thread with `std::thread::spawn`.
    pub fn run(mut self) {
        loop {
            let event = self.receiver.recv().unwrap();

            match event {
                Event::Terminate(callback) => {
                    callback.call(());
                    return;
                }
                Event::VerifyToTarget(height, callback) => {
                    let outcome = self.verify_to_target(height);
                    callback.call(outcome);
                }
                Event::VerifyToHighest(callback) => {
                    let outcome = self.verify_to_highest();
                    callback.call(outcome);
                }
                _ => {
                    // TODO: Log/record unexpected event
                }
            }
        }
    }
}

/// A handle to a `Supervisor` which allows to communicate with
/// the supervisor across thread boundaries via message passing.
pub struct Handle {
    sender: channel::Sender<Event>,
}

impl Handle {
    /// Crate a new handle that sends events to the supervisor via
    /// the given channel. For internal use only.
    pub fn new(sender: channel::Sender<Event>) -> Self {
        Self { sender }
    }

    /// Verify to the highest block.
    pub fn verify_to_highest(&mut self) -> VerificationResult {
        self.verify(Event::VerifyToHighest)
    }

    /// Verify to the block at the given height.
    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        self.verify(|callback| Event::VerifyToTarget(height, callback))
    }

    /// Verify either to the latest block (if `height == None`) or to a given block (if `height == Some(height)`).
    fn verify(
        &mut self,
        make_event: impl FnOnce(Callback<VerificationResult>) -> Event,
    ) -> VerificationResult {
        let (sender, receiver) = channel::bounded::<Event>(1);

        let callback = Callback::new(move |result| {
            // We need to create an event here
            let event = match result {
                Ok(header) => Event::VerificationSuccess(Box::new(header)),
                Err(err) => Event::VerificationFailure(err),
            };

            sender.send(event).unwrap();
        });

        let event = make_event(callback);
        self.sender.send(event).unwrap();

        match receiver.recv().unwrap() {
            Event::VerificationSuccess(header) => Ok(*header),
            Event::VerificationFailure(err) => Err(err),
            _ => todo!(),
        }
    }

    /// Async version of `verify_to_highest`.
    ///
    /// The given `callback` will be called asynchronously with the
    /// verification result.
    pub fn verify_to_highest_async(
        &mut self,
        callback: impl FnOnce(VerificationResult) -> () + Send + 'static,
    ) {
        let event = Event::VerifyToHighest(Callback::new(callback));
        self.sender.send(event).unwrap();
    }

    /// Async version of `verify_to_target`.
    ///
    /// The given `callback` will be called asynchronously with the
    /// verification result.
    pub fn verify_to_target_async(
        &mut self,
        height: Height,
        callback: impl FnOnce(VerificationResult) -> () + Send + 'static,
    ) {
        let event = Event::VerifyToTarget(height, Callback::new(callback));
        self.sender.send(event).unwrap();
    }

    /// Terminate the underlying supervisor.
    pub fn terminate(&mut self) {
        let (sender, receiver) = channel::bounded::<Event>(1);

        let callback = Callback::new(move |_| {
            sender.send(Event::Terminated).unwrap();
        });

        self.sender.send(Event::Terminate(callback)).unwrap();

        while let Ok(event) = receiver.recv() {
            match event {
                Event::Terminated => return,
                _ => continue,
            }
        }
    }
}
