use crate::prelude::*;

#[derive(Debug)]
pub enum Verdict {
    Success,
    NotEnoughTrust(VerificationError),
    Invalid(VerificationError),
}

impl Verdict {
    pub fn and_then(self, other: impl Fn() -> Verdict) -> Self {
        match self {
            Verdict::Success => other(),
            _ => self,
        }
    }
}

impl From<Result<(), VerificationError>> for Verdict {
    fn from(result: Result<(), VerificationError>) -> Self {
        match result {
            Ok(()) => Self::Success,
            Err(e) if e.not_enough_trust() => Self::NotEnoughTrust(e),
            Err(e) => Self::Invalid(e),
        }
    }
}

pub trait Verifier {
    fn verify(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &Options,
    ) -> Verdict {
        self.validate_light_block(light_block, trusted_state, options)
            .and_then(|| self.verify_overlap(light_block, trusted_state, options))
            .and_then(|| self.has_sufficient_voting_power(light_block, options))
    }

    fn validate_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &Options,
    ) -> Verdict;

    fn verify_overlap(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &Options,
    ) -> Verdict;

    fn has_sufficient_voting_power(&self, light_block: &LightBlock, options: &Options) -> Verdict;
}

pub struct ProdVerifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl ProdVerifier {
    pub fn new(
        predicates: impl VerificationPredicates + 'static,
        voting_power_calculator: impl VotingPowerCalculator + 'static,
        commit_validator: impl CommitValidator + 'static,
        header_hasher: impl HeaderHasher + 'static,
    ) -> Self {
        Self {
            predicates: Box::new(predicates),
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            header_hasher: Box::new(header_hasher),
        }
    }
}

impl Verifier for ProdVerifier {
    fn validate_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &Options,
    ) -> Verdict {
        crate::predicates::validate_light_block(
            &*self.predicates,
            &self.commit_validator,
            &self.header_hasher,
            &trusted_state,
            &light_block,
            options,
        )
        .into()
    }

    fn verify_overlap(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &Options,
    ) -> Verdict {
        crate::predicates::verify_overlap(
            &*self.predicates,
            &self.voting_power_calculator,
            &trusted_state,
            &light_block,
            options,
        )
        .into()
    }

    fn has_sufficient_voting_power(&self, light_block: &LightBlock, options: &Options) -> Verdict {
        crate::predicates::has_sufficient_voting_power(
            &*self.predicates,
            &self.voting_power_calculator,
            &light_block,
            options,
        )
        .into()
    }
}
