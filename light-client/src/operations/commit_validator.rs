use std::marker::PhantomData;

use crate::{
    bail,
    predicates::errors::VerificationError,
    types::{LightBlock, TMCommit, TMLightBlock, TMValidatorSet},
};

use tendermint::block::CommitSig;
use tendermint::lite::types::ValidatorSet as _;

pub trait CommitValidator<LB: LightBlock>: Send {
    fn validate(
        &self,
        commit: &LB::Commit,
        validators: &LB::ValidatorSet,
    ) -> Result<(), VerificationError>;

    fn validate_full(
        &self,
        commit: &LB::Commit,
        validators: &LB::ValidatorSet,
    ) -> Result<(), VerificationError>;
}

#[derive(Copy, Clone)]
pub struct ProdCommitValidator<LB> {
    marker: PhantomData<LB>,
}

impl<LB> Default for ProdCommitValidator<LB> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl CommitValidator<TMLightBlock> for ProdCommitValidator<TMLightBlock> {
    fn validate(
        &self,
        commit: &TMCommit,
        validator_set: &TMValidatorSet,
    ) -> Result<(), VerificationError> {
        // TODO: `self.commit.block_id` cannot be zero in the same way as in Go
        //       Clarify if this another encoding related issue
        if commit.signatures.len() == 0 {
            bail!(VerificationError::ImplementationSpecific(
                "no signatures for commit".to_string()
            ));
        }

        if commit.signatures.len() != validator_set.validators().len() {
            bail!(VerificationError::ImplementationSpecific(format!(
                "pre-commit length: {} doesn't match validator length: {}",
                commit.signatures.len(),
                validator_set.validators().len()
            )));
        }

        Ok(())
    }

    // This check is only necessary if we do full verification (2/3)
    //
    // See https://github.com/informalsystems/tendermint-rs/issues/281
    //
    // It returns `ImplementationSpecific` error if it detects a signer
    // that is not present in the validator set
    fn validate_full(
        &self,
        commit: &TMCommit,
        validator_set: &TMValidatorSet,
    ) -> Result<(), VerificationError> {
        for commit_sig in commit.signatures.iter() {
            let validator_address = match commit_sig {
                CommitSig::BlockIDFlagAbsent => continue,
                CommitSig::BlockIDFlagCommit {
                    validator_address, ..
                } => validator_address,
                CommitSig::BlockIDFlagNil {
                    validator_address, ..
                } => validator_address,
            };

            if validator_set.validator(*validator_address) == None {
                bail!(VerificationError::ImplementationSpecific(format!(
                    "Found a faulty signer ({}) not present in the validator set ({})",
                    validator_address,
                    validator_set.hash()
                )));
            }
        }

        Ok(())
    }
}
