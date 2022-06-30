// use std::collections::HashMap;

// use ed25519_consensus::SigningKey;
// use tendermint::account::Id;
// use tendermint_validator::{
//     BasicServerConfig, FileStateProvider, GrpcSocket, SoftwareSigner, SoftwareSignerServer,
// };

use std::collections::HashMap;

use tendermint::chain;
use tendermint_validator::{
    BasicServerConfig, FileStateProvider, GrpcSocket, KMSServer, SoftwareSigner,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    let mut providers = HashMap::new();
    let signer = SoftwareSigner::generate_ed25519(rand_core::OsRng);
    let state_provider = FileStateProvider::new("/tmp/validator.json".into())
        .await
        .unwrap();
    providers.insert(
        chain::Id::try_from("test-chain-4bmabt").unwrap(),
        (signer, state_provider),
    );
    let config = BasicServerConfig::new(None, GrpcSocket::Unix("/tmp/validator.test".into()));
    let server = KMSServer::new(providers, config).await.unwrap();
    server.serve().await
}
