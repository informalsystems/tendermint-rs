//! Synchronized state tracking for Tendermint blockchain networks the KMS
//! interacts with.
//!
//! Double-signing protection is the primary purpose of this code (for now).

mod error;
pub mod hook;

pub use self::error::{StateError, StateErrorKind};
use crate::{
    error::{Error, ErrorKind::*},
    prelude::*,
};
use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde_json;
use std::{
    fs,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};
use tendermint::consensus;

/// State tracking for double signing prevention
pub struct State {
    consensus_state: consensus::State,
    state_file_path: PathBuf,
}

impl State {
    /// Load the state from the given path
    pub fn load_state<P>(path: P) -> Result<State, Error>
    where
        P: AsRef<Path>,
    {
        let mut lst = State {
            consensus_state: consensus::State::default(),
            state_file_path: path.as_ref().to_owned(),
        };

        match fs::read_to_string(path.as_ref()) {
            Ok(contents) => {
                lst.consensus_state = serde_json::from_str(&contents).map_err(|e| {
                    err!(
                        ParseError,
                        "error parsing {}: {}",
                        path.as_ref().display(),
                        e
                    )
                })?;
                Ok(lst)
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    lst.sync_to_disk()?;
                    Ok(lst)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Check and update the chain's height, round, and step
    pub fn update_consensus_state(
        &mut self,
        new_state: consensus::State,
    ) -> Result<(), StateError> {
        // TODO(tarcieri): rewrite this using `Ord` impl on `consensus::State`
        if new_state.height < self.consensus_state.height {
            fail!(
                StateErrorKind::HeightRegression,
                "last height:{} new height:{}",
                self.consensus_state.height,
                new_state.height
            );
        } else if new_state.height == self.consensus_state.height {
            if new_state.round < self.consensus_state.round {
                fail!(
                    StateErrorKind::RoundRegression,
                    "round regression at height:{} last round:{} new round:{}",
                    new_state.height,
                    self.consensus_state.round,
                    new_state.round
                )
            } else if new_state.round == self.consensus_state.round {
                if new_state.step < self.consensus_state.step {
                    fail!(
                        StateErrorKind::StepRegression,
                        "round regression at height:{} round:{} last step:{} new step:{}",
                        new_state.height,
                        new_state.round,
                        self.consensus_state.step,
                        new_state.step
                    )
                }

                if new_state.block_id.is_some()
                    && self.consensus_state.block_id.is_some()
                    && self.consensus_state.block_id != new_state.block_id
                {
                    fail!(
                        StateErrorKind::DoubleSign,
                        "Attempting to sign a second proposal at height:{} round:{} step:{} old block id:{} new block {}",
                        new_state.height,
                        new_state.round,
                        new_state.step,
                        self.consensus_state.block_id.as_ref().unwrap(),
                        new_state.block_id.unwrap()
                    )
                }
            }
        }

        self.consensus_state = new_state;

        self.sync_to_disk().map_err(|e| {
            err!(
                StateErrorKind::SyncError,
                "error writing state to {}: {}",
                self.state_file_path.display(),
                e
            )
        })?;
        Ok(())
    }

    /// Update the internal state from the output from a hook command
    pub fn update_from_hook_output(&mut self, output: hook::Output) -> Result<(), StateError> {
        let hook_height = output.latest_block_height.value();
        let last_height = self.consensus_state.height.value();

        if hook_height > last_height {
            let delta = hook_height - last_height;

            if delta < hook::BLOCK_HEIGHT_SANITY_LIMIT {
                let mut new_state = consensus::State::default();
                new_state.height = output.latest_block_height;
                self.consensus_state = new_state;

                info!("updated block height from hook: {}", hook_height);
            } else {
                warn!(
                    "hook block height more than sanity limit: {} (delta: {}, max: {})",
                    output.latest_block_height,
                    delta,
                    hook::BLOCK_HEIGHT_SANITY_LIMIT
                );
            }
        } else {
            warn!(
                "hook block height less than current? current: {}, hook: {}",
                last_height, hook_height
            );
        }

        Ok(())
    }

    /// Sync the current state to disk
    fn sync_to_disk(&mut self) -> std::io::Result<()> {
        let json = serde_json::to_string(&self.consensus_state)?;

        AtomicFile::new(&self.state_file_path, OverwriteBehavior::AllowOverwrite)
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
            consensus_state: consensus::State {
                height: 1i64.into(),
                round: 1,
                step: 0,
                block_id: None,
            },
            state_file_path: EXAMPLE_PATH.into(),
        };

        assert_eq!(
            last_sign_state
                .update_consensus_state(consensus::State {
                    height: 2i64.into(),
                    round: 0,
                    step: 0,
                    block_id: None
                })
                .unwrap(),
            ()
        )
    }

    #[test]
    fn hrs_test_double_sign() {
        let mut last_sign_state = State {
            consensus_state: consensus::State {
                height: 1i64.into(),
                round: 1,
                step: 0,
                block_id: Some(block::Id::from_str(EXAMPLE_BLOCK_ID).unwrap()),
            },
            state_file_path: EXAMPLE_PATH.into(),
        };
        let double_sign_block = block::Id::from_str(EXAMPLE_DOUBLE_SIGN_BLOCK_ID).unwrap();
        let err = last_sign_state.update_consensus_state(consensus::State {
            height: 1i64.into(),
            round: 1,
            step: 1,
            block_id: Some(double_sign_block),
        });

        let double_sign_error = StateErrorKind::DoubleSign;

        assert_eq!(
            err.expect_err("Expect Double Sign error").0.kind(),
            &double_sign_error
        )
    }
}
