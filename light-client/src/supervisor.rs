use contracts::pre;
use crossbeam_channel as channel;

use tendermint::evidence::{ConflictingHeadersEvidence, Evidence};

use crate::{
    bail,
    callback::Callback,
    errors::{Error, ErrorKind},
    evidence::EvidenceReporter,
    fork_detector::{Fork, ForkDetection, ForkDetector},
    light_client::LightClient,
    peer_list::PeerList,
    state::State,
    types::{Height, LightBlock, PeerId, Status},
};

/// Input events sent by the [`Handle`]s to the [`Supervisor`]. They carry a [`Callback`] which is
/// used to communicate back the responses of the requests.
#[derive(Debug)]
enum HandleInput {
    /// Terminate the supervisor process
    Terminate(Callback<()>),
    /// Verify to the highest height, call the provided callback with result
    VerifyToHighest(Callback<Result<LightBlock, Error>>),
    /// Verify to the given height, call the provided callback with result
    VerifyToTarget(Height, Callback<Result<LightBlock, Error>>),
    /// Get the latest trusted block.
    LatestTrusted(Callback<Result<Option<LightBlock>, Error>>),
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

    pub fn latest_trusted(&self) -> Option<LightBlock> {
        self.state.light_store.highest(Status::Trusted)
    }

    pub fn trust_block(&mut self, lb: &LightBlock) {
        self.state.light_store.update(lb, Status::Trusted);
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
pub struct Supervisor {
    /// List of peers (primary + witnesses)
    peers: PeerList,
    /// An instance of the fork detector
    fork_detector: Box<dyn ForkDetector>,
    /// Reporter of fork evidence
    evidence_reporter: Box<dyn EvidenceReporter>,
    /// Channel through which to reply to `Handle`s
    sender: channel::Sender<HandleInput>,
    /// Channel through which to receive events from the `Handle`s
    receiver: channel::Receiver<HandleInput>,
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
        let (sender, receiver) = channel::unbounded::<HandleInput>();

        Self {
            peers,
            sender,
            receiver,
            fork_detector: Box::new(fork_detector),
            evidence_reporter: Box::new(evidence_reporter),
        }
    }

    /// Create a new handle to this supervisor.
    pub fn handle(&mut self) -> Handle {
        Handle::new(self.sender.clone())
    }

    #[pre(self.peers.primary().is_some())]
    fn latest_trusted(&self) -> Result<Option<LightBlock>, Error> {
        let primary = self.peers.primary().ok_or_else(|| ErrorKind::NoPrimary)?;
        Ok(primary.latest_trusted())
    }

    /// Verify to the highest block.
    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_highest(&mut self) -> Result<LightBlock, Error> {
        self.verify(None)
    }

    /// Verify to the block at the given height.
    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_target(&mut self, height: Height) -> Result<LightBlock, Error> {
        self.verify(Some(height))
    }

    /// Verify either to the latest block (if `height == None`) or to a given block (if `height == Some(height)`).
    #[pre(self.peers.primary().is_some())]
    fn verify(&mut self, height: Option<Height>) -> Result<LightBlock, Error> {
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
                // Verification succeeded, let's perform fork detection
                Ok(light_block) => {
                    let trusted_state = primary
                        .latest_trusted()
                        .ok_or_else(|| ErrorKind::NoTrustedState(Status::Trusted))?;

                    // Perform fork detection with the highest verified block as the trusted state.
                    let outcome = self.detect_forks(&light_block, &trusted_state)?;

                    match outcome {
                        // There was a fork or a faulty peer
                        ForkDetection::Detected(forks) => {
                            let forked = self.process_forks(forks)?;
                            if !forked.is_empty() {
                                // Fork detected, exiting
                                bail!(ErrorKind::ForkDetected(forked))
                            }
                        }
                        ForkDetection::NotDetected => {
                            // We need to re-ask for the primary here as the compiler
                            // is not smart enough to realize that we do not mutate
                            // the `primary` field of `PeerList` between the initial
                            // borrow of the primary and here (can't blame it, it's
                            // not that obvious).
                            // Note: This always succeeds since we already have a primary,
                            if let Some(primary) = self.peers.primary_mut() {
                                primary.trust_block(&light_block);
                            }

                            // No fork detected, exiting
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

    fn process_forks(&mut self, forks: Vec<Fork>) -> Result<Vec<PeerId>, Error> {
        let mut forked = Vec::with_capacity(forks.len());

        for fork in forks {
            match fork {
                // An actual fork was detected, report evidence and record forked peer.
                Fork::Forked { primary, witness } => {
                    let provider = witness.provider;
                    self.report_evidence(provider, &primary, &witness)?;

                    forked.push(provider);
                }
                // A witness has timed out, remove it from the peer list.
                Fork::Timeout(provider, _error) => {
                    self.peers.mark_witness_as_faulty(provider);
                    // TODO: Log/record the error
                }
                // A witness has been deemed faulty, remove it from the peer list.
                Fork::Faulty(block, _error) => {
                    self.peers.mark_witness_as_faulty(block.provider);
                    // TODO: Log/record the error
                }
            }
        }

        Ok(forked)
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
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
    ) -> Result<ForkDetection, Error> {
        if self.peers.witnesses().is_empty() {
            bail!(ErrorKind::NoWitnesses);
        }

        self.fork_detector
            .detect_forks(light_block, &trusted_state, self.peers.witnesses())
    }

    /// Run the supervisor event loop in the same thread.
    ///
    /// This method should typically be called within a new thread with `std::thread::spawn`.
    pub fn run(mut self) {
        loop {
            let event = self.receiver.recv().unwrap();

            match event {
                HandleInput::LatestTrusted(callback) => {
                    let outcome = self.latest_trusted();
                    callback.call(outcome);
                }
                HandleInput::Terminate(callback) => {
                    callback.call(());
                    return;
                }
                HandleInput::VerifyToTarget(height, callback) => {
                    let outcome = self.verify_to_target(height);
                    callback.call(outcome);
                }
                HandleInput::VerifyToHighest(callback) => {
                    let outcome = self.verify_to_highest();
                    callback.call(outcome);
                }
            }
        }
    }
}

/// A handle to a `Supervisor` which allows to communicate with
/// the supervisor across thread boundaries via message passing.
pub struct Handle {
    sender: channel::Sender<HandleInput>,
}

impl Handle {
    /// Crate a new handle that sends events to the supervisor via
    /// the given channel. For internal use only.
    fn new(sender: channel::Sender<HandleInput>) -> Self {
        Self { sender }
    }

    /// Get latest trusted block from the [`Supervisor`].
    pub fn latest_trusted(&mut self) -> Result<Option<LightBlock>, Error> {
        let (sender, receiver) = channel::bounded::<Result<Option<LightBlock>, Error>>(1);

        let callback = Callback::new(move |result| {
            sender.send(result).unwrap();
        });

        // TODO(xla): Transform crossbeam errors into proper domain errors.
        self.sender
            .send(HandleInput::LatestTrusted(callback))
            .unwrap();

        // TODO(xla): Transform crossbeam errors into proper domain errors.
        receiver.recv().unwrap()
    }

    /// Verify to the highest block.
    pub fn verify_to_highest(&mut self) -> Result<LightBlock, Error> {
        self.verify(HandleInput::VerifyToHighest)
    }

    /// Verify to the block at the given height.
    pub fn verify_to_target(&mut self, height: Height) -> Result<LightBlock, Error> {
        self.verify(|callback| HandleInput::VerifyToTarget(height, callback))
    }

    /// Verify either to the latest block (if `height == None`) or to a given block (if `height == Some(height)`).
    fn verify(
        &mut self,
        make_event: impl FnOnce(Callback<Result<LightBlock, Error>>) -> HandleInput,
    ) -> Result<LightBlock, Error> {
        let (sender, receiver) = channel::bounded::<Result<LightBlock, Error>>(1);

        let callback = Callback::new(move |result| {
            sender.send(result).unwrap();
        });

        let event = make_event(callback);
        self.sender.send(event).unwrap();

        receiver.recv().unwrap()
    }

    /// Async version of `verify_to_highest`.
    ///
    /// The given `callback` will be called asynchronously with the
    /// verification result.
    pub fn verify_to_highest_async(
        &mut self,
        callback: impl FnOnce(Result<LightBlock, Error>) -> () + Send + 'static,
    ) {
        let event = HandleInput::VerifyToHighest(Callback::new(callback));
        self.sender.send(event).unwrap();
    }

    /// Async version of `verify_to_target`.
    ///
    /// The given `callback` will be called asynchronously with the
    /// verification result.
    pub fn verify_to_target_async(
        &mut self,
        height: Height,
        callback: impl FnOnce(Result<LightBlock, Error>) -> () + Send + 'static,
    ) {
        let event = HandleInput::VerifyToTarget(height, Callback::new(callback));
        self.sender.send(event).unwrap();
    }

    /// Terminate the underlying supervisor.
    pub fn terminate(&mut self) {
        let (sender, receiver) = channel::bounded::<()>(1);

        let callback = Callback::new(move |_| {
            sender.send(()).unwrap();
        });

        self.sender.send(HandleInput::Terminate(callback)).unwrap();

        receiver.recv().unwrap()
    }
}
