//! Validator gRPC high-level server implementation

use std::collections::BTreeMap;

use tendermint::{
    block, chain, consensus,
    proposal::{SignProposalRequest, SignedProposalResponse},
    public_key::{PubKeyRequest, PubKeyResponse},
    vote::{SignVoteRequest, SignedVoteResponse},
    PublicKey, Signature,
};
use tendermint_proto::privval::{
    priv_validator_api_server::{PrivValidatorApi, PrivValidatorApiServer},
    PubKeyRequest as RawPubKeyRequest, PubKeyResponse as RawPubKeyResponse, RemoteSignerError,
    SignProposalRequest as RawSignProposalRequest, SignVoteRequest as RawSignVoteRequest,
    SignedProposalResponse as RawSignedProposalResponse,
    SignedVoteResponse as RawSignedVoteResponse,
};
use tokio::{net::UnixListener, sync::Mutex, time::Instant};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error, info};

use crate::{
    config::{BasicServerConfig, GrpcSocket},
    signer::{display_validator_info, SignerProvider},
    state::ValidatorStateProvider,
};

/// Validator gRPC high-level server implementation
#[derive(Debug)]
pub struct PrivvalService<S: SignerProvider, VS: ValidatorStateProvider> {
    /// Signer and state providers for individual chains.
    /// Given the traits require mutable access to the state provider
    /// across awaits, we use tokio's mutex here.
    providers: Mutex<BTreeMap<chain::Id, (S, VS)>>,
    /// A cache of public keys loaded during the initialization.
    pubkeys: BTreeMap<chain::Id, PublicKey>,
    /// Optional setting for the maximum height,
    /// after which the server will stop signing for a particular network.
    max_heights: BTreeMap<chain::Id, block::Height>,
    /// The network configuration for the gRPC server.
    config: BasicServerConfig,
}

fn check_state(
    chain_id: &chain::Id,
    current: &consensus::State,
    next: &consensus::State,
) -> Result<(), RemoteSignerError> {
    if next > current {
        Ok(())
    } else {
        error!(
            "[{}] attempted double sign at h/r/s: {} ({} != {})",
            chain_id,
            next,
            current.block_id_prefix(),
            next.block_id_prefix()
        );
        Err(get_double_sign_error(&next.height))
    }
}

impl<S: SignerProvider, VS: ValidatorStateProvider> PrivvalService<S, VS> {
    /// Creates a new server instance by loading all providers.
    pub async fn new(providers: BTreeMap<chain::Id, (S, VS)>, config: BasicServerConfig) -> Self {
        info!("creating a new KMS server");
        let mut pubkeys = BTreeMap::new();
        for (chain_id, (signer, _)) in providers.iter() {
            let pubkey = signer.verifying_key();
            let (address, pubkeyb64) = display_validator_info(pubkey);
            info!("[{}] loaded a validator ID: {}", chain_id, address);
            info!("[{}] public key: {}", chain_id, pubkeyb64);
            pubkeys.insert(chain_id.clone(), *pubkey);
        }

        Self {
            providers: Mutex::new(providers),
            pubkeys,
            max_heights: BTreeMap::new(),
            config,
        }
    }

    fn check_max_height(&self, chain_id: &chain::Id, new_height: block::Height) -> Result<(), ()> {
        match self.max_heights.get(chain_id) {
            Some(max_height) if new_height > *max_height => Err(()),
            _ => Ok(()),
        }
    }
}

impl<S, VS> PrivvalService<S, VS>
where
    S: SignerProvider + 'static,
    VS: ValidatorStateProvider + Sync + Send + 'static,
{
    /// Based on the connection configuration, starts the gRPC server.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut server = Server::builder();
        let config = self.config.clone();
        if let Some(tls_config) = config.tls_config {
            debug!("configuring TLS");
            server = server.tls_config(tls_config)?;
        }
        let router = server.add_service(PrivValidatorApiServer::new(self));
        match config.socket {
            GrpcSocket::Tcp(addr) => {
                info!(
                    "starting the Tendermint KMS gRPC server to listen on TCP: {}",
                    addr
                );
                router.serve(addr).await?;
            },
            GrpcSocket::Unix(path) => {
                info!(
                    "starting the Tendermint KMS gRPC server to listen on Unix Domain Socket: {}",
                    path.display()
                );
                let uds = UnixListener::bind(path)?;
                let uds_stream = UnixListenerStream::new(uds);
                router.serve_with_incoming(uds_stream).await?;
            },
        }
        Ok(())
    }
}

async fn sign_and_persist_state<S, VS>(
    chain_id: &chain::Id,
    signer: &S,
    state_provider: &mut VS,
    new_state: consensus::State,
    signable_bytes: Vec<u8>,
) -> Result<Signature, RemoteSignerError>
where
    S: SignerProvider + 'static,
    VS: ValidatorStateProvider + Sync + Send + 'static,
{
    let state = state_provider.load_state().await.map_err(|e| {
        error!("[{}] failed to load the existing state: {}", chain_id, e);
        get_state_not_found_error()
    })?;
    check_state(chain_id, &state, &new_state)?;
    let started_at = Instant::now();
    let signature = signer.sign_async(&signable_bytes).await.map_err(|e| {
        error!("[{}] failed to sign: {}", chain_id, e);
        get_failed_to_sign_error()
    })?;
    info!(
        "[{}] signed: {} at h/r/s {} ({} ms)",
        chain_id,
        new_state.block_id_prefix(),
        new_state,
        started_at.elapsed().as_millis(),
    );
    if let Err(e) = state_provider.persist_state(&new_state).await {
        error!("[{}] failed to persist the state: {}", chain_id, e);
    }
    Ok(signature)
}

/// raw error types for the responses
fn get_invalid_chain_id_error(chain_id: &chain::Id) -> RemoteSignerError {
    RemoteSignerError {
        code: 1,
        description: format!("invalid chain id: {}", chain_id),
    }
}

/// raw error types for the responses
fn get_double_sign_error(height: &block::Height) -> RemoteSignerError {
    RemoteSignerError {
        code: 2,
        description: format!("double signing requested at height: {}", height),
    }
}

/// raw error types for the responses
fn get_state_not_found_error() -> RemoteSignerError {
    RemoteSignerError {
        code: 3,
        description: "existing state failed to load (internal error)".to_owned(),
    }
}

/// raw error types for the responses
fn get_failed_to_sign_error() -> RemoteSignerError {
    RemoteSignerError {
        code: 4,
        description: "signer failed to sign (internal error)".to_owned(),
    }
}

#[tonic::async_trait]
impl<S, VS> PrivValidatorApi for PrivvalService<S, VS>
where
    S: SignerProvider + 'static,
    VS: ValidatorStateProvider + Sync + Send + 'static,
{
    async fn get_pub_key(
        &self,
        request: Request<RawPubKeyRequest>,
    ) -> Result<Response<RawPubKeyResponse>, Status> {
        let req = PubKeyRequest::try_from(request.into_inner())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        debug!("[{}] received a public key request", req.chain_id);
        let resp = match self.pubkeys.get(&req.chain_id) {
            Some(pubkey) => PubKeyResponse {
                pub_key: Some(*pubkey),
                error: None,
            },
            None => {
                error!("[{}] no public key found", req.chain_id);
                PubKeyResponse {
                    pub_key: None,
                    error: Some(get_invalid_chain_id_error(&req.chain_id)),
                }
            },
        };
        Ok(Response::new(resp.into()))
    }
    async fn sign_vote(
        &self,
        request: Request<RawSignVoteRequest>,
    ) -> Result<Response<RawSignedVoteResponse>, Status> {
        let req = SignVoteRequest::try_from(request.into_inner())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        debug!(
            "[{}] received a vote signing request: {:?}",
            req.chain_id, req
        );
        self.check_max_height(&req.chain_id, req.vote.height)
            .map_err(|_| Status::failed_precondition("max height exceeded"))?;
        let mut providers = self.providers.lock().await;
        let resp: SignedVoteResponse;
        if let Some((signer, state_provider)) = providers.get_mut(&req.chain_id) {
            let new_state = (&req).into();
            let signable_bytes = req
                .vote
                .to_signable_vec(req.chain_id.clone())
                .map_err(|e| Status::internal(e.to_string()))?;
            match sign_and_persist_state(
                &req.chain_id,
                signer,
                state_provider,
                new_state,
                signable_bytes,
            )
            .await
            {
                Ok(signature) => {
                    let mut vote = req.vote.clone();
                    vote.signature = Some(signature);
                    resp = SignedVoteResponse {
                        vote: Some(vote),
                        error: None,
                    };
                },
                Err(err) => {
                    resp = SignedVoteResponse {
                        vote: None,
                        error: Some(err),
                    };
                },
            }
        } else {
            error!("[{}] no signer found", req.chain_id);
            resp = SignedVoteResponse {
                vote: None,
                error: Some(get_invalid_chain_id_error(&req.chain_id)),
            };
        };
        Ok(Response::new(resp.into()))
    }
    async fn sign_proposal(
        &self,
        request: Request<RawSignProposalRequest>,
    ) -> Result<Response<RawSignedProposalResponse>, Status> {
        let req = SignProposalRequest::try_from(request.into_inner())
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        debug!(
            "[{}] received a proposal signing request: {:?}",
            req.chain_id, req
        );
        self.check_max_height(&req.chain_id, req.proposal.height)
            .map_err(|_| Status::failed_precondition("max height exceeded"))?;
        let mut providers = self.providers.lock().await;
        let resp: SignedProposalResponse;
        if let Some((signer, state_provider)) = providers.get_mut(&req.chain_id) {
            let new_state = (&req).into();
            let signable_bytes = req
                .proposal
                .to_signable_vec(req.chain_id.clone())
                .map_err(|e| Status::internal(e.to_string()))?;
            match sign_and_persist_state(
                &req.chain_id,
                signer,
                state_provider,
                new_state,
                signable_bytes,
            )
            .await
            {
                Ok(signature) => {
                    let mut proposal = req.proposal.clone();
                    proposal.signature = Some(signature);
                    resp = SignedProposalResponse {
                        proposal: Some(proposal),
                        error: None,
                    };
                },
                Err(err) => {
                    resp = SignedProposalResponse {
                        proposal: None,
                        error: Some(err),
                    };
                },
            }
        } else {
            error!("[{}] no signer found", req.chain_id);
            resp = SignedProposalResponse {
                proposal: None,
                error: Some(get_invalid_chain_id_error(&req.chain_id)),
            };
        };
        Ok(Response::new(resp.into()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tendermint::{block::Height, chain, consensus, proposal::Type, Proposal, Vote};
    use tendermint_proto::privval::priv_validator_api_server::PrivValidatorApi;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::{
        generate_ed25519, BasicServerConfig, GrpcSocket, PrivvalService, SoftwareSigner,
        ValidatorStateProvider,
    };

    const CHAIN_ID: &str = "test";
    const CHAIN_ID2: &str = "test2";

    #[derive(Default)]
    pub struct MockStateProvider {
        last_state: consensus::State,
    }

    #[tonic::async_trait]
    impl ValidatorStateProvider for MockStateProvider {
        type E = crate::error::Error;

        async fn load_state(&self) -> Result<consensus::State, Self::E> {
            Ok(self.last_state.clone())
        }

        async fn persist_state(&mut self, new_state: &consensus::State) -> Result<(), Self::E> {
            self.last_state = new_state.clone();
            Ok(())
        }
    }

    async fn test_setup() -> PrivvalService<SoftwareSigner, MockStateProvider> {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
        let mut providers = BTreeMap::new();
        let signer = generate_ed25519(rand_core::OsRng);
        let state_provider = MockStateProvider::default();
        providers.insert(
            chain::Id::try_from(CHAIN_ID).unwrap(),
            (signer, state_provider),
        );
        let config = BasicServerConfig::new(None, GrpcSocket::Unix("/tmp/test.socket".into()));
        PrivvalService::new(providers, config).await
    }

    #[tokio::test]
    pub async fn test_get_pubkey() {
        let server = test_setup().await;
        let req = tonic::Request::new(super::RawPubKeyRequest {
            chain_id: CHAIN_ID.to_string(),
        });
        let resp = server.get_pub_key(req).await.unwrap();
        let resp = resp.into_inner();
        assert!(resp.pub_key.is_some() && resp.error.is_none());
        let req2 = tonic::Request::new(super::RawPubKeyRequest {
            chain_id: CHAIN_ID2.to_string(),
        });
        let resp2 = server.get_pub_key(req2).await.unwrap();
        let resp2 = resp2.into_inner();
        assert!(resp2.pub_key.is_none() && resp2.error.is_some());
        assert_eq!(resp2.error.unwrap().code, 1);
    }

    #[tokio::test]
    pub async fn test_sign_vote() {
        let server = test_setup().await;
        let vote = Vote::default();
        let req = tonic::Request::new(super::RawSignVoteRequest {
            chain_id: CHAIN_ID.to_string(),
            vote: Some(vote.clone().into()),
        });
        let resp = server.sign_vote(req).await.unwrap();
        let resp = resp.into_inner();
        assert!(resp.vote.is_some() && resp.error.is_none());
        let req2 = tonic::Request::new(super::RawSignVoteRequest {
            chain_id: CHAIN_ID2.to_string(),
            vote: Some(vote.into()),
        });
        let resp2 = server.sign_vote(req2).await.unwrap();
        let resp2 = resp2.into_inner();
        assert!(resp2.vote.is_none() && resp2.error.is_some());
        assert_eq!(resp2.error.unwrap().code, 1);
    }

    #[tokio::test]
    pub async fn test_sign_proposal() {
        let server = test_setup().await;
        let proposal = Proposal {
            msg_type: Type::Proposal,
            height: Height::increment(Default::default()),
            round: Default::default(),
            pol_round: None,
            block_id: None,
            timestamp: None,
            signature: None,
        };
        let req = tonic::Request::new(super::RawSignProposalRequest {
            chain_id: CHAIN_ID.to_string(),
            proposal: Some(proposal.clone().into()),
        });
        let resp = server.sign_proposal(req).await.unwrap();
        let resp = resp.into_inner();
        assert!(resp.proposal.is_some() && resp.error.is_none());
        let req2 = tonic::Request::new(super::RawSignProposalRequest {
            chain_id: CHAIN_ID2.to_string(),
            proposal: Some(proposal.into()),
        });
        let resp2 = server.sign_proposal(req2).await.unwrap();
        let resp2 = resp2.into_inner();
        assert!(resp2.proposal.is_none() && resp2.error.is_some());
        assert_eq!(resp2.error.unwrap().code, 1);
    }

    #[tokio::test]
    pub async fn test_double_sign() {
        let server = test_setup().await;
        let vote = Vote::default();
        let inner_req = super::RawSignVoteRequest {
            chain_id: CHAIN_ID.to_string(),
            vote: Some(vote.clone().into()),
        };
        let req = tonic::Request::new(inner_req.clone());
        let resp = server.sign_vote(req).await.unwrap();
        let resp = resp.into_inner();
        assert!(resp.vote.is_some() && resp.error.is_none());
        let req2 = tonic::Request::new(inner_req);
        let resp2 = server.sign_vote(req2).await.unwrap();
        let resp2 = resp2.into_inner();
        assert!(resp2.vote.is_none() && resp2.error.is_some());
        assert_eq!(resp2.error.unwrap().code, 2);
    }
}
