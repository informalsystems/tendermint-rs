//! Provides an interface and default implementation for the `CommitValidator` operation

use tendermint::block::CommitSig;

use crate::{
    errors::VerificationError,
    types::{SignedHeader, ValidatorSet},
};

/// Validates the commit associated with a header against a validator set
pub trait CommitValidator: Send + Sync {
    /// Perform basic validation
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        let signatures = &signed_header.commit.signatures;

        // Check the the commit contains at least one non-absent signature.
        // See https://github.com/informalsystems/tendermint-rs/issues/650
        let has_present_signatures = signatures.iter().any(|cs| !cs.is_absent());
        if !has_present_signatures {
            return Err(VerificationError::no_signature_for_commit());
        }

        // Check that that the number of signatures matches the number of validators.
        if signatures.len() != validator_set.validators().len() {
            return Err(VerificationError::mismatch_pre_commit_length(
                signatures.len(),
                validator_set.validators().len(),
            ));
        }

        Ok(())
    }

    /// Perform full validation, only necessary if we do full verification (2/3)
    fn validate_full(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), VerificationError> {
        for commit_sig in signed_header.commit.signatures.iter() {
            let validator_address = match commit_sig {
                CommitSig::BlockIdFlagAbsent => continue,
                CommitSig::BlockIdFlagCommit {
                    validator_address, ..
                } => validator_address,
                CommitSig::BlockIdFlagNil {
                    validator_address, ..
                } => validator_address,
            };

            if validator_set.validator(*validator_address).is_none() {
                return Err(VerificationError::faulty_signer(
                    *validator_address,
                    validator_set.clone(),
                ));
            }
        }

        Ok(())
    }
}

/// Production-ready implementation of a commit validator.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct ProdCommitValidator;

impl CommitValidator for ProdCommitValidator {}
