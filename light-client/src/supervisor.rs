use crate::{
    bail,
    callback::Callback,
    components::io::IoError,
    errors::{Error, ErrorKind},
    fork_detector::{Fork, ForkDetection, ForkDetector},
    light_client::LightClient,
    peer_list::PeerList,
    state::State,
    store::VerifiedStatus,
    types::{Height, LightBlock},
};

use contracts::pre;
use crossbeam_channel as channel;

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
    pub fn new(peers: PeerList, fork_detector: impl ForkDetector + 'static) -> Self {
        let (sender, receiver) = channel::unbounded::<Event>();

        Self {
            peers,
            sender,
            receiver,
            fork_detector: Box::new(fork_detector),
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
                                        self.peers.remove_witness(&block.provider);
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

        bail!(ErrorKind::NoWitnessLeft)
    }

    fn report_evidence(&mut self, _light_block: &LightBlock) {
        ()
    }

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
                ErrorKind::Io(IoError::Timeout(peer)) => {
                    self.peers.remove_witness(peer);
                    Err(e)
                }
                _ => Err(e),
            },
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
