use crate::callback::Callback;
use crate::fork_detector::{Fork, ForkDetection, ForkDetector, ProdForkDetector};
use crate::prelude::*;

use contracts::pre;
use crossbeam_channel as channel;
use std::collections::HashMap;

pub type VerificationResult = Result<LightBlock, Error>;

#[derive(Debug)]
pub enum Event {
    Terminate(Callback<()>),
    Terminated,
    VerifyToHighest(Callback<VerificationResult>),
    VerifyToTarget(Height, Callback<VerificationResult>),
    VerificationSuccessed(LightBlock),
    VerificationFailed(Error),
}

#[derive(Default)]
pub struct PeerListBuilder {
    primary: Option<PeerId>,
    peers: HashMap<PeerId, Instance>,
}

impl PeerListBuilder {
    pub fn primary(mut self, primary: PeerId) -> Self {
        self.primary = Some(primary);
        self
    }

    pub fn peer(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.peers.insert(peer_id, instance);
        self
    }

    #[pre(
        self.primary.is_some() && self.peers.contains_key(self.primary.as_ref().unwrap())
    )]
    pub fn build(self) -> PeerList {
        PeerList {
            primary: self.primary.unwrap(),
            peers: self.peers,
        }
    }
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

#[derive(Debug)]
pub struct PeerList {
    peers: HashMap<PeerId, Instance>,
    primary: PeerId,
}

impl PeerList {
    pub fn builder() -> PeerListBuilder {
        PeerListBuilder::default()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&Instance> {
        self.peers.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Instance> {
        self.peers.get_mut(peer_id)
    }

    pub fn primary(&self) -> Option<&Instance> {
        self.peers.get(&self.primary)
    }

    pub fn primary_mut(&mut self) -> Option<&mut Instance> {
        self.peers.get_mut(&self.primary)
    }

    pub fn secondaries(&self) -> Vec<&Instance> {
        self.peers
            .keys()
            .filter(|peer_id| peer_id != &&self.primary)
            .filter_map(|peer_id| self.get(peer_id))
            .collect()
    }

    #[pre(peer_id != &self.primary)]
    pub fn remove_secondary(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }

    pub fn swap_primary(&mut self) -> Result<(), Error> {
        if let Some(peer_id) = self.peers.keys().next() {
            if peer_id != &self.primary {
                self.primary = *peer_id;
                return Ok(());
            }
        }

        bail!(ErrorKind::NoValidPeerLeft)
    }
}

#[derive(Debug)]
pub struct Supervisor {
    peers: PeerList,
    sender: channel::Sender<Event>,
    receiver: channel::Receiver<Event>,
}

// Ensure the `Supervisor` can be sent across thread boundaries.
static_assertions::assert_impl_all!(Supervisor: Send);

impl Supervisor {
    pub fn new(peers: PeerList) -> Self {
        let (sender, receiver) = channel::unbounded::<Event>();

        Self {
            sender,
            receiver,
            peers,
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

                    let outcome = self.detect_forks(&light_block, &trusted_state);

                    match outcome {
                        Some(forks) => {
                            let mut forked = Vec::with_capacity(forks.len());

                            for fork in forks {
                                match fork {
                                    Fork::Forked(block) => {
                                        self.report_evidence(&block);
                                        forked.push(block.provider);
                                    }
                                    Fork::Faulty(block) => {
                                        self.peers.remove_secondary(&block.provider);
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
                    // Swap primary, and continue with new primary, if any
                    self.peers.swap_primary()?;
                    continue;
                }
            }
        }

        bail!(ErrorKind::NoValidPeerLeft)
    }

    fn report_evidence(&mut self, _light_block: &LightBlock) {
        ()
    }

    #[pre(self.peers.primary().is_some())]
    fn detect_forks(
        &mut self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
    ) -> Option<Vec<Fork>> {
        if self.peers.secondaries().is_empty() {
            return None;
        }

        let fork_detector = ProdForkDetector::new(); // TODO: Should be injectable
        let result =
            fork_detector.detect_forks(light_block, &trusted_state, self.peers.secondaries());

        match result {
            ForkDetection::Detected(forks) => Some(forks),
            ForkDetection::NotDetected => None,
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
                    // NoOp?
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
        let (sender, receiver) = channel::bounded::<Event>(1);

        let callback = Callback::new(move |result| {
            // We need to create an event here
            let event = match result {
                Ok(header) => Event::VerificationSuccessed(header),
                Err(err) => Event::VerificationFailed(err),
            };

            sender.send(event).unwrap();
        });

        self.sender.send(Event::VerifyToHighest(callback)).unwrap();

        match receiver.recv().unwrap() {
            Event::VerificationSuccessed(header) => Ok(header),
            Event::VerificationFailed(err) => Err(err),
            _ => todo!(),
        }
    }

    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        let (sender, receiver) = channel::bounded::<Event>(1);

        let callback = Callback::new(move |result| {
            // We need to create an event here
            let event = match result {
                Ok(header) => Event::VerificationSuccessed(header),
                Err(err) => Event::VerificationFailed(err),
            };

            sender.send(event).unwrap();
        });

        self.sender
            .send(Event::VerifyToTarget(height, callback))
            .unwrap();

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
