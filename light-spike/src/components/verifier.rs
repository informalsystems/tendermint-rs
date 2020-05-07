use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Verdict {
    Success,
    NotEnoughTrust,
    Invalid(VerificationError),
}

pub trait Verifier {
    fn validate_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> Verdict;

    fn verify_overlap(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> Verdict;

    fn has_sufficient_voting_power(
        &self,
        light_block: &LightBlock,
        options: &VerificationOptions,
    ) -> Verdict;
}

pub struct RealVerifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl RealVerifier {
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

impl Verifier for RealVerifier {
    fn validate_light_block(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> Verdict {
        let result = crate::predicates::validate_light_block(
            &*self.predicates,
            &self.commit_validator,
            &self.header_hasher,
            &trusted_state,
            &light_block,
            options,
        );

        match result {
            Ok(()) => Verdict::Success,
            Err(e) => Verdict::Invalid(e),
        }
    }

    fn verify_overlap(
        &self,
        light_block: &LightBlock,
        trusted_state: &TrustedState,
        options: &VerificationOptions,
    ) -> Verdict {
        let result = crate::predicates::verify_overlap(
            &*self.predicates,
            &self.voting_power_calculator,
            &trusted_state,
            &light_block,
            options,
        );

        match result {
            Ok(()) => Verdict::Success,
            Err(e) => Verdict::Invalid(e),
        }
    }

    fn has_sufficient_voting_power(
        &self,
        light_block: &LightBlock,
        options: &VerificationOptions,
    ) -> Verdict {
        let result = crate::predicates::has_sufficient_voting_power(
            &*self.predicates,
            &self.voting_power_calculator,
            &light_block,
            options,
        );

        match result {
            Ok(()) => Verdict::Success,
            Err(_) => Verdict::NotEnoughTrust,
        }
    }
}
