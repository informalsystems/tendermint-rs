use abscissa::Error;
use serde_json;
use std::{
    fmt::{self, Display},
    fs::File,
    io::prelude::*,
    path::Path,
};
use tendermint::{block, chain};

#[derive(Serialize, Deserialize)]
struct LastSignData {
    pub height: i64,
    pub round: i64,
    pub step: i8,
    pub block_id: Option<block::Id>,
}

const EMPTY_DATA: LastSignData = LastSignData {
    height: 0,
    round: 0,
    step: 0,
    block_id: None,
};

pub struct LastSignState {
    data: LastSignData,
    file: File,
    _chain_id: chain::Id,
}

/// Error type
#[derive(Debug)]
pub struct LastSignError(Error<LastSignErrorKind>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LastSignErrorKind {
    #[fail(display = "height regression")]
    HeightRegression,
    #[fail(display = "step regression")]
    StepRegression,
    #[fail(display = "round regression")]
    RoundRegression,
    #[fail(display = "invalid block id")]
    DoubleSign,
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
    pub fn load_state(path: &Path, chain_id: chain::Id) -> std::io::Result<LastSignState> {
        if !path.exists() {
            let mut lst = LastSignState {
                data: EMPTY_DATA,
                file: File::create(path)?,
                _chain_id: chain_id,
            };
            lst.sync_to_disk()?;
            return Ok(lst);
        }
        let mut lst = LastSignState {
            data: EMPTY_DATA,
            file: File::open(path)?,
            _chain_id: chain_id,
        };

        let mut contents = String::new();
        lst.file.read_to_string(&mut contents)?;
        lst.data = serde_json::from_str(&contents).unwrap();
        Ok(lst)
    }

    pub fn sync_to_disk(&mut self) -> std::io::Result<()> {
        self.file
            .write_all(serde_json::to_string(&self.data).unwrap().as_ref())?;
        self.file.sync_all()?;
        Ok(())
    }

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
        }
        if height == self.data.height {
            if round < self.data.round {
                fail!(
                    LastSignErrorKind::RoundRegression,
                    "round regression at height:{} last round:{} new round:{}",
                    height,
                    self.data.round,
                    round
                )
            }
            if round == self.data.round {
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

                if block_id != None && self.data.block_id != None && self.data.block_id != block_id
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
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tendermint::{block, chain};

    const EXAMPLE_BLOCK_ID: &str =
        "26C0A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    const EXAMPLE_DOUBLE_SIGN_BLOCK_ID: &str =
        "2470A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    #[test]
    fn hrs_test() {
        let mut last_sign_state = LastSignState {
            data: LastSignData {
                height: 1,
                round: 1,
                step: 0,
                block_id: None,
            },
            file: File::create("/tmp/tmp_state.json").unwrap(),
            _chain_id: "example-chain".parse::<chain::Id>().unwrap(),
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
            file: File::create("/tmp/tmp_state.json").unwrap(),
            _chain_id: "example-chain".parse::<chain::Id>().unwrap(),
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
