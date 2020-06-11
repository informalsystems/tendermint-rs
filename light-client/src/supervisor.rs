use std::collections::HashMap;

use crate::callback::Callback;
use crate::fork_detector::{Fork, ForkDetection, ForkDetector};
use crate::peer_list::PeerList;
use crate::prelude::*;

use contracts::{contract_trait, pre};
use crossbeam_channel as channel;
use tendermint::rpc;

pub type VerificationResult = Result<LightBlock, Error>;

#[derive(Debug)]
pub enum Event {
    // Inputs
    Terminate(Callback<()>),
    VerifyToHighest(Callback<VerificationResult>),
    VerifyToTarget(Height, Callback<VerificationResult>),

    // Outputs
    Terminated,
    VerificationSuccessed(LightBlock),
    VerificationFailed(Error),
}

#[derive(Debug)]
pub struct Instance {
    pub light_client: LightClient,
    pub state: State,
}

impl Instance {
    pub fn new(light_client: LightClient, state: State) -> Self {
        Self {
            light_client,
            state,
        }
    }
}

pub struct Supervisor {
    peers: PeerList,
    fork_detector: Box<dyn ForkDetector>,
    sender: channel::Sender<Event>,
    receiver: channel::Receiver<Event>,
    evidence_reporter: Box<dyn EvidenceReporter>,
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
    pub fn new(peers: PeerList,
        fork_detector: impl ForkDetector + 'static,
        evidence_reporter: impl EvidenceReporter + 'static) -> Self {
        let (sender, receiver) = channel::unbounded::<Event>();

        Self {
            peers,
            sender,
            receiver,
            fork_detector: Box::new(fork_detector),
            evidence_reporter: Box::new(evidence_reporter),
        }
    }

    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_highest(&mut self) -> VerificationResult {
        self.verify(None)
    }

    #[pre(self.peers.primary().is_some())]
    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        self.verify(Some(height))
    }

    #[pre(self.peers.primary().is_some())]
    fn verify(&mut self, height: Option<Height>) -> VerificationResult {
        while let Some(primary) = self.peers.primary_mut() {
            let verdict = match height {
                None => primary.light_client.verify_to_highest(&mut primary.state),
                Some(height) => primary
                    .light_client
                    .verify_to_target(height, &mut primary.state),
            };

            match verdict {
                Ok(light_block) => {
                    // SAFETY: There must be a latest trusted state otherwise verification would have failed.
                    let trusted_state = primary
                        .state
                        .light_store
                        .latest(VerifiedStatus::Verified)
                        .unwrap();

                    let outcome = self.detect_forks(&light_block, &trusted_state)?;

                    match outcome {
                        Some(forks) => {
                            let mut forked = Vec::with_capacity(forks.len());

                            for fork in forks {
                                match fork {
                                    Fork::Forked(block) => {
                                        self.report_evidence(&block);
                                        forked.push(block.provider);
                                    }
                                    Fork::Faulty(block, _error) => {
                                        self.peers.remove_secondary(&block.provider);
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
                    // Swap primary, and continue with new primary, if any.
                    self.peers.swap_primary()?;
                    // TODO: Log/record error
                    continue;
                }
            }
        }

        bail!(ErrorKind::NoValidPeerLeft)
    }

    fn report_evidence(&mut self, _light_block: &LightBlock) {
        // self.evidence_reporter.report()
    }

    #[pre(self.peers.primary().is_some())]
    fn detect_forks(
        &mut self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
    ) -> Result<Option<Vec<Fork>>, Error> {
        if self.peers.secondaries().is_empty() {
            return Ok(None);
        }

        let result = self.fork_detector.detect_forks(
            light_block,
            &trusted_state,
            self.peers.secondaries(),
        )?;

        match result {
            ForkDetection::Detected(forks) => Ok(Some(forks)),
            ForkDetection::NotDetected => Ok(None),
        }
    }

    pub fn handle(&mut self) -> Handle {
        Handle::new(self.sender.clone())
    }

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

pub struct Handle {
    sender: channel::Sender<Event>,
}

impl Handle {
    pub fn new(sender: channel::Sender<Event>) -> Self {
        Self { sender }
    }

    pub fn verify_to_highest(&mut self) -> VerificationResult {
        self.verify(Event::VerifyToHighest)
    }

    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        self.verify(|callback| Event::VerifyToTarget(height, callback))
    }

    fn verify(
        &mut self,
        make_event: impl FnOnce(Callback<VerificationResult>) -> Event,
    ) -> VerificationResult {
        let (sender, receiver) = channel::bounded::<Event>(1);

        let callback = Callback::new(move |result| {
            // We need to create an event here
            let event = match result {
                Ok(header) => Event::VerificationSuccessed(header),
                Err(err) => Event::VerificationFailed(err),
            };

            sender.send(event).unwrap();
        });

        let event = make_event(callback);
        self.sender.send(event).unwrap();

        match receiver.recv().unwrap() {
            Event::VerificationSuccessed(header) => Ok(header),
            Event::VerificationFailed(err) => Err(err),
            _ => todo!(),
        }
    }

    pub fn verify_to_highest_async(
        &mut self,
        callback: impl FnOnce(VerificationResult) -> () + Send + 'static,
    ) {
        let event = Event::VerifyToHighest(Callback::new(callback));
        self.sender.send(event).unwrap();
    }

    pub fn verify_to_target_async(
        &mut self,
        height: Height,
        callback: impl FnOnce(VerificationResult) -> () + Send + 'static,
    ) {
        let event = Event::VerifyToTarget(height, Callback::new(callback));
        self.sender.send(event).unwrap();
    }

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

struct Evidence {

}

/// Interface for reporting evidence to full nodes, typically via the RPC client.
#[contract_trait]
pub trait EvidenceReporter: Send {
    /// Report evidence to all connected full nodes.
    fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError>;
}

/// Production implementation of the EvidenceReporter component, which reports evidence to full
/// nodes via RPC.
#[derive(Clone, Debug)]
pub struct ProdEvidenceReporter {
    peer_map: HashMap<PeerId, tendermint::net::Address>,
}

#[contract_trait]
impl EvidenceReporter for ProdEvidenceReporter {

    #[pre(self.peer_map.contains_key(&peer))]
    fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError> {
        let res = block_on(self.rpc_client_for(peer).broadcast_evidence(e));

        match res {
            Ok(response) => Ok(response.hash),
            Err(err) => Err(IoError::IoError(err)),
        }
    }
}

impl ProdEvidenceReporter {
    /// Constructs a new ProdEvidenceReporter component.
    ///
    /// A peer map which maps peer IDS to their network address must be supplied.
    pub fn new(peer_map: HashMap<PeerId, tendermint::net::Address>) -> Self {
        Self { peer_map }
    }

    // FIXME: Cannot enable precondition because of "autoref lifetime" issue
    // #[pre(self.peer_map.contains_key(&peer))]
    fn rpc_client_for(&self, peer: PeerId) -> rpc::Client {
        let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
        rpc::Client::new(peer_addr)
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
