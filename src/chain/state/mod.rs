//! Synchronized state tracking for Tendermint blockchain networks the KMS
//! interacts with.
//!
//! Double-signing protection is the primary purpose of this code (for now).

pub mod last_sign;

pub use self::last_sign::{LastSignError, LastSignErrorKind};
use crate::{chain, error::KmsError};
use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde_json;
use std::{
    fs,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};
use tendermint::block;

/// Get mutex guarded access to the current state of a particular chain
pub fn synchronize<F, T>(chain_id: chain::Id, func: F) -> Result<T, KmsError>
where
    F: Fn(&mut State) -> Result<T, KmsError>,
{
    let registry = chain::REGISTRY.get();
    let chain = registry
        .chain(chain_id)
        .unwrap_or_else(|| panic!("can't update state for unregistered chain: {}", chain_id));

    // TODO(tarcieri): better handle `PoisonError`?
    let mut state_guard = chain.state.lock().unwrap();
    func(&mut state_guard)
}

/// State tracking for double signing prevention
pub struct State {
    data: last_sign::Data,
    path: PathBuf,
}

impl State {
    /// Load the state from the given path
    pub fn load_state<P>(path: P) -> std::io::Result<State>
    where
        P: AsRef<Path>,
    {
        let mut lst = State {
            data: last_sign::Data::default(),
            path: path.as_ref().to_owned(),
        };

        match fs::read_to_string(path) {
            Ok(contents) => {
                lst.data = serde_json::from_str(&contents)?;
                Ok(lst)
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    lst.sync_to_disk()?;
                    Ok(lst)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Check and update the chain's height, round, and step
    pub fn check_and_update_hrs(
        &mut self,
        height: i64,
        round: i64,
        step: i8,
        block_id: Option<block::Id>,
    ) -> Result<(), LastSignError> {
        if height < self.data.height {
            fail!(
                LastSignErrorKind::HeightRegression,
                "last height:{} new height:{}",
                self.data.height,
                height
            );
        } else if height == self.data.height {
            if round < self.data.round {
                fail!(
                    LastSignErrorKind::RoundRegression,
                    "round regression at height:{} last round:{} new round:{}",
                    height,
                    self.data.round,
                    round
                )
            } else if round == self.data.round {
                if step < self.data.step {
                    fail!(
                        LastSignErrorKind::StepRegression,
                        "round regression at height:{} round:{} last step:{} new step:{}",
                        height,
                        round,
                        self.data.step,
                        step
                    )
                }

                if block_id.is_some()
                    && self.data.block_id.is_some()
                    && self.data.block_id != block_id
                {
                    fail!(
                        LastSignErrorKind::DoubleSign,
                        "Attempting to sign a second proposal at height:{} round:{} step:{} old block id:{} new block {}",
                        height,
                        round,
                        step,
                        self.data.block_id.unwrap(),
                        block_id.unwrap()
                    )
                }
            }
        }

        self.data.height = height;
        self.data.round = round;
        self.data.step = step;
        self.data.block_id = block_id;

        self.sync_to_disk().map_err(|e| {
            err!(
                LastSignErrorKind::SyncError,
                "error writing state to {}: {}",
                self.path.display(),
                e
            )
        })?;
        Ok(())
    }

    /// Sync the current state to disk
    fn sync_to_disk(&mut self) -> std::io::Result<()> {
        let json = serde_json::to_string(&self.data)?;

        AtomicFile::new(&self.path, OverwriteBehavior::AllowOverwrite)
            .write(|f| f.write_all(json.as_bytes()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tendermint::block;

    const EXAMPLE_BLOCK_ID: &str =
        "26C0A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    const EXAMPLE_DOUBLE_SIGN_BLOCK_ID: &str =
        "2470A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    const EXAMPLE_PATH: &str = "/tmp/tmp_state.json";

    #[test]
    fn hrs_test() {
        let mut last_sign_state = State {
            data: last_sign::Data {
                height: 1,
                round: 1,
                step: 0,
                block_id: None,
            },
            path: EXAMPLE_PATH.into(),
        };
        assert_eq!(
            last_sign_state.check_and_update_hrs(2, 0, 0, None).unwrap(),
            ()
        )
    }

    #[test]
    fn hrs_test_double_sign() {
        let mut last_sign_state = State {
            data: last_sign::Data {
                height: 1,
                round: 1,
                step: 0,
                block_id: Some(block::Id::from_str(EXAMPLE_BLOCK_ID).unwrap()),
            },
            path: EXAMPLE_PATH.into(),
        };
        let double_sign_block = block::Id::from_str(EXAMPLE_DOUBLE_SIGN_BLOCK_ID).unwrap();
        let err = last_sign_state.check_and_update_hrs(1, 1, 1, Some(double_sign_block));

        let double_sign_error = LastSignErrorKind::DoubleSign;

        assert_eq!(
            err.expect_err("Expect Double Sign error").0.kind(),
            &double_sign_error
        )
    }
}
