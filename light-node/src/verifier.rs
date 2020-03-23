use crate::requester::RPCRequester;
use futures::stream::StreamExt;
use std::time::{Duration, SystemTime};
use tendermint::block::signed_header::SignedHeader as TMCommit;
use tendermint::block::Header as TMHeader;
use tendermint::lite::error::Kind;
use tendermint::lite::{
    verify_single_inner, Header, Height, Requester, SignedHeader, TrustThresholdFraction,
    TrustedState,
};
use tendermint::validator::Set;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub struct VerificationRequest {
    trusted_state: TrustedState<TMCommit, TMHeader>,
    untrusted_height: Height,
    trust_threshold: TrustThresholdFraction,
    trusting_period: Duration,
    now: SystemTime,
    result_sender: oneshot::Sender<Vec<TrustedState<TMCommit, TMHeader>>>,
}

pub struct Verifier {
    verification_request_sender: mpsc::Sender<VerificationRequest>,
}

impl Verifier {
    pub fn new(requester: RPCRequester) -> Verifier {
        let (verification_request_sender, receiver) = mpsc::channel(1);
        let mut receiver = receiver.fuse();

        tokio::spawn(async move {
            loop {
                select! {
                    new_request = receiver.select_next_some() => {
                        let VerificationRequest {
                            trusted_state,
                            untrusted_height,
                            trust_threshold,
                            trusting_period,
                            now,
                            result_sender,
                        } = new_request;

                        let mut bisection_verifier = BisectionVerifier {
                            requester: requester.clone(),
                            trusted_left: None,
                            untrusted_height: Some(untrusted_height),
                            trusted_state: Some(trusted_state),
                            pivot_height: None,
                            trust_threshold,
                            untrusted_sh: None,
                            untrusted_vals: None,
                            untrusted_next_vals: None,
                            cache: None,
                        };

                        tokio::spawn(async move {
                            loop {
                                let _ = bisection_verifier.update();
                                if let Some(bisection_results) = bisection_verifier.verify() {
                                    let _ = result_sender.send(bisection_results);
                                    break;
                                }
                            }
                        });
                    }
                    complete => break,
                }
            }
        });

        Verifier {
            verification_request_sender,
        }
    }

    pub async fn verify_bisection(
        &mut self,
        trusted_state: TrustedState<TMCommit, TMHeader>,
        untrusted_height: Height,
        trust_threshold: TrustThresholdFraction,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Vec<TrustedState<TMCommit, TMHeader>> {
        let (result_sender, receiver) = oneshot::channel();
        let _ = self
            .verification_request_sender
            .send(VerificationRequest {
                trusted_state,
                untrusted_height,
                trust_threshold,
                trusting_period,
                now,
                result_sender,
            })
            .await;
        receiver.await.expect("Failed to get verification result.")
    }
}

pub struct BisectionVerifier {
    requester: RPCRequester,
    trusted_left: Option<TrustedState<TMCommit, TMHeader>>,
    untrusted_height: Option<Height>,
    trusted_state: Option<TrustedState<TMCommit, TMHeader>>,
    pivot_height: Option<Height>,
    trust_threshold: TrustThresholdFraction,
    untrusted_sh: Option<SignedHeader<TMCommit, TMHeader>>,
    untrusted_vals: Option<Set>,
    untrusted_next_vals: Option<Set>,
    cache: Option<Vec<TrustedState<TMCommit, TMHeader>>>,
}

impl BisectionVerifier {
    pub async fn update(&mut self) {
        let current_untrusted_height = match self.pivot_height.as_ref() {
            Some(pivot) => pivot,
            None => self
                .untrusted_height
                .as_ref()
                .expect("No untrusted height present."),
        };

        self.untrusted_sh = Some(
            self.requester
                .signed_header(*current_untrusted_height)
                .await
                .expect(""),
        );

        self.untrusted_vals = Some(
            self.requester
                .validator_set(*current_untrusted_height)
                .await
                .expect(""),
        );
        self.untrusted_next_vals = Some(
            self.requester
                .validator_set(
                    current_untrusted_height
                        .checked_add(1)
                        .expect("height overflow"),
                )
                .await
                .expect(""),
        );
    }

    pub fn verify(&mut self) -> Option<Vec<TrustedState<TMCommit, TMHeader>>> {
        let (current_untrusted_height, current_trusted_state) = match self.pivot_height.take() {
            Some(pivot) => (pivot, self.trusted_state.take()),
            None => {
                let height = self
                    .untrusted_height
                    .take()
                    .expect("No untrusted height present.");
                let state = self.trusted_left.take();
                (height, state)
            }
        };

        let current_trusted_state = match current_trusted_state {
            Some(state) => state,
            None => {
                // Terminate.
                return self.cache.take();
            }
        };

        match verify_single_inner(
            &current_trusted_state,
            self.untrusted_sh.as_ref().expect(""),
            self.untrusted_vals.as_ref().expect(""),
            self.untrusted_next_vals.as_ref().expect(""),
            self.trust_threshold,
        ) {
            Ok(_) => {
                // Successfully verified!
                let signed_headers = self.untrusted_sh.take().expect("");
                let next_validators = self.untrusted_next_vals.take().expect("");
                let ts = TrustedState::new(signed_headers, next_validators);
                self.trusted_left = Some(ts.clone());
                self.untrusted_height = Some(current_untrusted_height);
                let mut cache = self.cache.take().unwrap_or(vec![]);
                cache.push(ts);
                self.cache = Some(cache);
                None
            }
            Err(e) => {
                match e.kind() {
                    // Insufficient voting power to update.
                    // Engage bisection.
                    &Kind::InsufficientVotingPower { .. } => {
                        // Get the pivot height for bisection.
                        let trusted_h = current_trusted_state.last_header().header().height();
                        let untrusted_h = current_untrusted_height;
                        self.pivot_height =
                            Some(trusted_h.checked_add(untrusted_h).expect("height overflow") / 2);
                        self.trusted_state = Some(current_trusted_state);
                        None
                    }
                    // If something went wrong, terminate.
                    _ => None,
                }
            }
        }
    }
}
