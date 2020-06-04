use crate::callback::Callback;
use crate::prelude::*;

use contracts::pre;
use crossbeam_channel as channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type VerificationResult = Result<LightBlock, Error>;

#[derive(Debug)]
pub enum Event {
    Terminate(Callback<()>),
    Terminated,
    VerifyToTarget(Height, Callback<VerificationResult>),
    VerificationSuccessed(LightBlock),
    VerificationFailed(Error),
}

#[derive(Default)]
pub struct PeerListBuilder {
    primary: Option<PeerId>,
    peers: HashMap<PeerId, LightClient>,
}

impl PeerListBuilder {
    pub fn primary(&mut self, primary: PeerId) -> &mut Self {
        self.primary = Some(primary);
        self
    }

    pub fn peer(&mut self, peer_id: PeerId, client: LightClient) -> &mut Self {
        self.peers.insert(peer_id, client);
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
pub struct PeerList {
    peers: HashMap<PeerId, LightClient>,
    primary: PeerId,
}

impl PeerList {
    pub fn builder() -> PeerListBuilder {
        PeerListBuilder::default()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&LightClient> {
        self.peers.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut LightClient> {
        self.peers.get_mut(peer_id)
    }

    pub fn primary(&self) -> Option<&LightClient> {
        self.peers.get(&self.primary)
    }

    pub fn primary_mut(&mut self) -> Option<&mut LightClient> {
        self.peers.get_mut(&self.primary)
    }

    pub fn secondaries(&self) -> Vec<&LightClient> {
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
            self.primary = *peer_id;
            return Ok(());
        }

        bail!(ErrorKind::NoValidPeerLeft)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Fork {
    PassedVerification(PeerId),
    FailedVerification(PeerId),
}

#[derive(Debug)]
pub struct Supervisor {
    peers: PeerList,
    sender: channel::Sender<Event>,
    receiver: channel::Receiver<Event>,
}

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
    pub fn verify_to_target(&mut self, height: Height) -> VerificationResult {
        while let Some(primary) = self.peers.primary_mut() {
            let verdict = primary.verify_to_target(height);

            match verdict {
                Ok(light_block) => {
                    let outcome = self.detect_forks(&light_block);
                    match outcome {
                        Some(forks) => {
                            let mut forked = Vec::with_capacity(forks.len());

                            for fork in forks {
                                match fork {
                                    Fork::PassedVerification(peer_id) => {
                                        self.report_evidence(&light_block);
                                        forked.push(peer_id);
                                    }
                                    Fork::FailedVerification(peer_id) => {
                                        self.peers.remove_secondary(&peer_id);
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
    fn detect_forks(&mut self, light_block: &LightBlock) -> Option<Vec<Fork>> {
        use crate::fork_detector::{ForkDetector, ProdForkDetector};

        if self.peers.secondaries().is_empty() {
            return None;
        }

        let primary = self.peers.primary().unwrap();
        let secondaries = self.peers.secondaries();

        let fork_detector = ProdForkDetector::new();
        let _result = fork_detector.detect_forks(light_block, primary, secondaries);
        Some(todo())
    }

    pub fn handler(&mut self) -> Handler {
        Handler::new(self.sender.clone())
    }

    // Consume the instance here but return a runtime which will allow interaction
    // Maybe return an output channnel here?
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
                _ => {
                    // NoOp?
                }
            }
        }
    }
}

pub struct Handler {
    sender: channel::Sender<Event>,
}

// Assume single handler
impl Handler {
    // How do we connect with the runtime?
    pub fn new(sender: channel::Sender<Event>) -> Self {
        Self { sender }
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
            Event::VerificationFailed(_err) => todo!(),
            _ => todo!(),
        }
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

#[allow(dead_code, unused_variables)]
mod test {
    use super::*;

    fn test(mut s: Supervisor) {
        let h1 = s.handler();
        let h2 = s.handler();

        std::thread::spawn(move || s.run());
    }
}
