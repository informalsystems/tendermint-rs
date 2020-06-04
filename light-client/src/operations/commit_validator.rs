use crate::prelude::*;

use anomaly::BoxError;
use dyn_clone::DynClone;
use tendermint::lite::types::Commit as _;

pub trait CommitValidator: Send + DynClone {
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), BoxError>;
}

impl<T: CommitValidator + Send + Sync> CommitValidator for &T {
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<(), BoxError> {
        (*self).validate(signed_header, validators)
    }
}

// impl CommitValidator for Box<dyn CommitValidator> {
//     fn validate(
//         &self,
//         signed_header: &SignedHeader,
//         validators: &ValidatorSet,
//     ) -> Result<(), BoxError> {
//         self.as_ref().validate(signed_header, validators)
//     }
// }

#[derive(Copy, Clone)]
pub struct ProdCommitValidator;

impl CommitValidator for ProdCommitValidator {
    fn validate(
        &self,
        signed_header: &SignedHeader,
        validator_set: &ValidatorSet,
    ) -> Result<(), BoxError> {
        // TODO: self.commit.block_id cannot be zero in the same way as in go
        // clarify if this another encoding related issue
        if signed_header.commit.signatures.len() == 0 {
            bail!(VerificationError::ImplementationSpecific(
                "no signatures for commit".to_string()
            ));
        }
        if signed_header.commit.signatures.len() != validator_set.validators().len() {
            bail!(VerificationError::ImplementationSpecific(format!(
                "pre-commit length: {} doesn't match validator length: {}",
                signed_header.commit.signatures.len(),
                validator_set.validators().len()
            )));
        }

        signed_header.validate(&validator_set)?;

        Ok(())
    }
}
