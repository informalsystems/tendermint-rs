//! Validator state-related interface and the sample file-based implementation.

use std::{io::Write, path::PathBuf};

use tempfile::NamedTempFile;
use tendermint::consensus;
use tracing::debug;

use crate::error::Error;

/// The trait for the validator state storage.
/// (file, external DB, monotonic CPU counters...)
#[tonic::async_trait]
pub trait ValidatorStateProvider {
    type E: std::error::Error;
    async fn load_state(&self) -> Result<consensus::State, Self::E>;
    async fn persist_state(&mut self, new_state: &consensus::State) -> Result<(), Self::E>;
}

/// The default file-based implementation of [`ValidatorStateProvider`].
pub struct FileStateProvider {
    state_file_path: PathBuf,
    last_state: consensus::State,
}

impl FileStateProvider {
    pub async fn new(state_file_path: PathBuf) -> Result<Self, Error> {
        match tokio::fs::read_to_string(&state_file_path).await {
            Ok(state_json) => {
                let consensus_state: consensus::State = serde_json::from_str(&state_json)
                    .map_err(|e| Error::json_error(state_file_path.display().to_string(), e))?;

                Ok(Self {
                    state_file_path,
                    last_state: consensus_state,
                })
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let consensus_state = consensus::State {
                    height: 0u32.into(),
                    ..Default::default()
                };
                let mut provider = Self {
                    state_file_path,
                    last_state: consensus_state.clone(),
                };
                provider.persist_state(&consensus_state).await?;
                Ok(provider)
            },
            Err(e) => Err(Error::io_error(state_file_path.display().to_string(), e)),
        }
    }
}

#[tonic::async_trait]
impl ValidatorStateProvider for FileStateProvider {
    type E = Error;
    async fn load_state(&self) -> Result<consensus::State, Error> {
        Ok(self.last_state.clone())
    }

    async fn persist_state(&mut self, new_state: &consensus::State) -> Result<(), Error> {
        debug!(
            "writing new consensus state to {}: {:?}",
            self.state_file_path.display(),
            &new_state
        );

        let json = serde_json::to_string(&new_state)
            .map_err(|e| Error::json_error(self.state_file_path.display().to_string(), e))?;

        let state_file_dir = self.state_file_path.parent().unwrap_or_else(|| {
            panic!("state file cannot be root directory");
        });

        let mut state_file = NamedTempFile::new_in(state_file_dir)
            .map_err(|e| Error::io_error(self.state_file_path.display().to_string(), e))?;
        state_file
            .write_all(json.as_bytes())
            .map_err(|e| Error::io_error(self.state_file_path.display().to_string(), e))?;
        state_file
            .persist(&self.state_file_path)
            .map_err(|e| Error::io_error(self.state_file_path.display().to_string(), e.error))?;

        debug!(
            "successfully wrote new consensus state to {}",
            self.state_file_path.display(),
        );

        self.last_state = new_state.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{FileStateProvider, ValidatorStateProvider};

    #[tokio::test]
    pub async fn test_file_persistence() {
        let tf = tempfile::TempDir::new().expect("temp dir");
        let path = tf.path().join("validator.json");
        let mut provider = FileStateProvider::new(path.clone())
            .await
            .expect("file provider");
        let mut state = provider.load_state().await.expect("load state 1");
        state.height = state.height.increment();
        provider.persist_state(&state).await.expect("persist state");
        let state2 = provider.load_state().await.expect("load state 2");
        assert_eq!(state, state2);
        let provider = FileStateProvider::new(path.clone())
            .await
            .expect("file provider");
        let state3 = provider.load_state().await.expect("load state 3");
        assert_eq!(state, state3);
    }
}
