use crate::{chain, error::KmsError};
use abscissa::Error;
use atomicwrites::{AtomicFile, OverwriteBehavior};
use serde_json;
use std::{
    fmt::{self, Display},
    fs,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};
use tendermint::block;

/// Check and update the chain position for the given `chain::Id`
pub fn check_and_update_hrs(
    chain_id: chain::Id,
    height: i64,
    round: i64,
    step: i8,
    block_id: Option<block::Id>,
) -> Result<(), KmsError> {
    let registry = chain::REGISTRY.get();
    let chain = registry
        .chain(chain_id)
        .unwrap_or_else(|| panic!("can't update state for unregistered chain: {}", chain_id));

    // TODO(tarcieri): better handle `PoisonErrore`?
    let mut last_sign_state = chain.state.lock().unwrap();

    last_sign_state
        .check_and_update_hrs(height, round, step, block_id)
        .map_err(|e| {
            warn!("double sign event: {}", e);
            e
        })?;

    Ok(())
}

/// Position of the chain the last time we attempted to sign
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct LastSignData {
    pub height: i64,
    pub round: i64,
    pub step: i8,
    pub block_id: Option<block::Id>,
}

/// State tracking for double signing prevention
pub struct LastSignState {
    data: LastSignData,
    path: PathBuf,
}

/// Error type
#[derive(Debug)]
pub struct LastSignError(Error<LastSignErrorKind>);

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LastSignErrorKind {
    /// Height regressed
    #[fail(display = "height regression")]
    HeightRegression,

    /// Step regressed
    #[fail(display = "step regression")]
    StepRegression,

    /// Round regressed
    #[fail(display = "round regression")]
    RoundRegression,

    /// Double sign detected
    #[fail(display = "double sign detected")]
    DoubleSign,

    /// Error syncing state to disk
    #[fail(display = "error syncing state to disk")]
    SyncError,
}

impl From<Error<LastSignErrorKind>> for LastSignError {
    fn from(other: Error<LastSignErrorKind>) -> Self {
        LastSignError(other)
    }
}

impl Display for LastSignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl LastSignState {
    /// Load the state from the given path
    pub fn load_state<P>(path: P) -> std::io::Result<LastSignState>
    where
        P: AsRef<Path>,
    {
        let mut lst = LastSignState {
            data: LastSignData::default(),
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
                        self.data.block_id.clone().unwrap(),
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
        let mut last_sign_state = LastSignState {
            data: LastSignData {
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
        let mut last_sign_state = LastSignState {
            data: LastSignData {
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
