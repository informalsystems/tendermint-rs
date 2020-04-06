//! Sketch out the pseudo code for a light client
//! That integrates the learning from the last iteration.
//! What we want:
//! + Simple light client specific types, no crypto
//! + Crypto can abstracted into traits which implement crypto specific functions
//! + Express the core verification logic as a composition of predicates to allow mocking

#![allow(dead_code, unreachable_code)]

use derive_more::Display;
use std::time::{Duration, SystemTime};

use pred::inspect::Inspect;
use pred::*;

// Some simplified types which only have the fields needed for core verification

type Hash = u64;
type Height = u64;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InvalidCommit,
    InvalidValidatorSet,
    InvalidNextValidatorSet,
    InvalidCommitValue,
    ImplementationSpecific,
    NonIncreasingHeight,
    NonMonotonicBftTime,
    InsufficientVotingPower,
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct Header {
    height: Height,
    bft_time: SystemTime,
    validator_set_hash: Hash,
    next_validator_set_hash: Hash,
    hash: Hash, // What if we don't have this
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct ValidatorSet {
    hash: Hash,
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct Commit {
    header_hash: Hash,
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct TrustLevel {
    numerator: u64,
    denominator: u64,
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct SignedHeader {
    header: Header,
    commit: Commit,
    validators: ValidatorSet,
    validator_hash: Hash,
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{:?}", self)]
struct TrustedState {
    header: Header,
    validators: ValidatorSet,
}

// Crypto function traits allowing mocking out during testing
trait VotingPowerCalculator: Sized {
    // What kind of errors should we be reporting here?
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> Result<u64, Error>;
    fn total_power_of(&self, validators: &ValidatorSet) -> Result<u64, Error>;
}

trait CommitValidator: Sized {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), Error>;
}

trait HeaderHasher: Sized {
    fn hash(&self, header: &Header) -> Hash; // Or Error?
}

/// Predicates

fn _validator_sets_match(signed_header: &SignedHeader, validators: &ValidatorSet) -> bool {
    signed_header.validator_hash == validators.hash
}

fn validator_sets_match(
    signed_header: SignedHeader,
    validators: ValidatorSet,
) -> impl Predicate + Inspect {
    pred::from_fn(move || _validator_sets_match(&signed_header, &validators))
        .named("validator_sets_match")
}

fn _next_validators_match(signed_header: &SignedHeader, validators: &ValidatorSet) -> bool {
    signed_header.validator_hash == validators.hash
}

fn next_validators_match(
    signed_header: SignedHeader,
    validators: ValidatorSet,
) -> impl Predicate + Inspect {
    pred::from_fn(move || _next_validators_match(&signed_header, &validators))
        .named("next_validators_match")
}

fn _header_matches_commit(
    header: &Header,
    commit: &Commit,
    header_hasher: &impl HeaderHasher,
) -> bool {
    header_hasher.hash(header) == commit.header_hash
}

fn header_matches_commit(
    header: Header,
    commit: Commit,
    header_hasher: impl HeaderHasher,
) -> impl Predicate + Inspect {
    pred::from_fn(move || _header_matches_commit(&header, &commit, &header_hasher))
        .named("header_matches_commit")
}

fn _valid_commit(
    commit: &Commit,
    validators: &ValidatorSet,
    validator: &impl CommitValidator,
) -> bool {
    validator.validate(commit, validators).is_ok()
}

fn valid_commit(
    commit: Commit,
    validators: ValidatorSet,
    validator: impl CommitValidator,
) -> impl Predicate + Inspect {
    pred::from_fn(move || _valid_commit(&commit, &validators, &validator)).named("valid_commit")
}

fn _is_within_trusted_period(header: &Header, trusting_period: Duration, now: SystemTime) -> bool {
    let header_time: SystemTime = header.bft_time.into();
    let expires_at = header_time + trusting_period;

    header_time < now && expires_at > now
}

fn is_within_trusted_period(
    header: Header,
    trusting_period: Duration,
    now: SystemTime,
) -> impl Predicate + Inspect {
    pred::from_fn(move || _is_within_trusted_period(&header, trusting_period, now))
        .named("is_within_trusted_period")
}

fn _is_monotonic_bft_time(header_a: &Header, header_b: &Header) -> bool {
    header_b.bft_time >= header_a.bft_time
}

fn is_monotonic_bft_time(header_a: Header, header_b: Header) -> impl Predicate + Inspect {
    pred::from_fn(move || _is_monotonic_bft_time(&header_a, &header_b))
        .named("is_monotonic_bft_time")
}

fn _is_monotonic_height(header_a: &Header, header_b: &Header) -> bool {
    header_a.height > header_b.height
}

fn is_monotonic_height(header_a: Header, header_b: Header) -> impl Predicate + Inspect {
    pred::from_fn(move || _is_monotonic_height(&header_a, &header_b)).named("is_monotonic_height")
}

fn _has_sufficient_voting_power(
    commit: &Commit,
    validators: &ValidatorSet,
    trust_level: &TrustLevel,
    calculator: &impl VotingPowerCalculator,
) -> bool {
    let total_power = calculator.total_power_of(validators);
    let voting_power = calculator.voting_power_in(commit, validators);

    if let (Ok(total_power), Ok(voting_power)) = (total_power, voting_power) {
        // XXX: Maybe trust_level doesn't need a very sophisticated type
        voting_power * trust_level.denominator > total_power * trust_level.numerator
    } else {
        false
    }
}

fn has_sufficient_voting_power(
    commit: Commit,
    validators: ValidatorSet,
    trust_level: TrustLevel,
    calculator: impl VotingPowerCalculator,
) -> impl Predicate + Inspect {
    pred::from_fn(move || {
        _has_sufficient_voting_power(&commit, &validators, &trust_level, &calculator)
    })
    .named("has_sufficient_voting_power")
}

fn _has_sufficient_validators_overlap(
    untrusted_commit: &Commit,
    trusted_validators: &ValidatorSet,
    trust_level: &TrustLevel,
    calculator: &impl VotingPowerCalculator,
) -> bool {
    _has_sufficient_voting_power(
        untrusted_commit,
        trusted_validators,
        trust_level,
        calculator,
    )
}

fn has_sufficient_validators_overlap(
    untrusted_commit: Commit,
    trusted_validators: ValidatorSet,
    trust_level: TrustLevel,
    calculator: impl VotingPowerCalculator,
) -> impl Predicate + Inspect {
    pred::from_fn(move || {
        _has_sufficient_validators_overlap(
            &untrusted_commit,
            &trusted_validators,
            &trust_level,
            &calculator,
        )
    })
    .named("has_sufficient_validators_overlap")
}

fn _has_sufficient_signers_overlap(
    untrusted_commit: &Commit,
    untrusted_validators: &ValidatorSet,
    trust_level: &TrustLevel,
    calculator: &impl VotingPowerCalculator,
) -> bool {
    _has_sufficient_voting_power(
        untrusted_commit,
        untrusted_validators,
        trust_level,
        calculator,
    )
}

fn has_sufficient_signers_overlap(
    untrusted_commit: Commit,
    untrusted_validators: ValidatorSet,
    trust_level: TrustLevel,
    calculator: impl VotingPowerCalculator,
) -> impl Predicate + Inspect {
    pred::from_fn(move || {
        _has_sufficient_signers_overlap(
            &untrusted_commit,
            &untrusted_validators,
            &trust_level,
            &calculator,
        )
    })
    .named("has_sufficient_signers_overlap")
}

fn _invalid_next_validator_set(
    trusted_state: &TrustedState,
    untrusted_sh: &SignedHeader,
    untrusted_next_vals: &ValidatorSet,
) -> bool {
    untrusted_sh.header.height == trusted_state.header.height
        && trusted_state.validators.hash != untrusted_next_vals.hash
}

fn invalid_next_validator_set(
    trusted_state: TrustedState,
    untrusted_sh: SignedHeader,
    untrusted_next_vals: ValidatorSet,
) -> impl Predicate + Inspect {
    pred::from_fn(move || {
        _invalid_next_validator_set(&trusted_state, &untrusted_sh, &untrusted_next_vals)
    })
    .named("invalid_next_validator_set")
}

fn verify_pred(
    trusted_state: TrustedState,
    untrusted_sh: SignedHeader,
    untrusted_vals: ValidatorSet,
    untrusted_next_vals: ValidatorSet,
    trust_level: TrustLevel,

    // Operations
    validator: impl CommitValidator + Clone,
    calculator: impl VotingPowerCalculator + Clone,
    header_hasher: impl HeaderHasher + Clone,
) -> impl Predicate + Inspect {
    validator_sets_match(untrusted_sh.clone(), untrusted_vals.clone())
        .and(next_validators_match(
            untrusted_sh.clone(),
            untrusted_next_vals.clone(),
        ))
        .and(header_matches_commit(
            untrusted_sh.header.clone(),
            untrusted_sh.commit.clone(),
            header_hasher.clone(),
        ))
        .and(valid_commit(
            untrusted_sh.commit.clone(),
            untrusted_sh.validators.clone(),
            validator.clone(),
        ))
        .and(is_monotonic_bft_time(
            untrusted_sh.header.clone(),
            trusted_state.header.clone(),
        ))
        .and(is_monotonic_height(
            trusted_state.header.clone(),
            untrusted_sh.header.clone(),
        ))
        .and(not(invalid_next_validator_set(
            trusted_state.clone(),
            untrusted_sh.clone(),
            untrusted_next_vals.clone(),
        )))
        .and(has_sufficient_validators_overlap(
            untrusted_sh.commit.clone(),
            trusted_state.validators.clone(),
            trust_level.clone(),
            calculator.clone(),
        ))
        .and(has_sufficient_signers_overlap(
            untrusted_sh.commit.clone(),
            untrusted_vals.clone(),
            trust_level.clone(),
            calculator.clone(),
        ))
}

fn verify(
    trusted_state: TrustedState,
    untrusted_sh: SignedHeader,
    untrusted_vals: ValidatorSet,
    untrusted_next_vals: ValidatorSet,
    trust_level: TrustLevel,

    // Operations
    validator: impl CommitValidator + Clone,
    calculator: impl VotingPowerCalculator + Clone,
    header_hasher: impl HeaderHasher + Clone,
) -> Result<(), Error> {
    // shouldn't this return a new TrustedState?

    if !validator_sets_match(untrusted_sh.clone(), untrusted_vals.clone()).eval() {
        return Err(Error::InvalidValidatorSet);
    }

    if !next_validators_match(untrusted_sh.clone(), untrusted_next_vals.clone()).eval() {
        return Err(Error::InvalidNextValidatorSet);
    }

    if !header_matches_commit(
        untrusted_sh.header.clone(),
        untrusted_sh.commit.clone(),
        header_hasher.clone(),
    )
    .eval()
    {
        return Err(Error::InvalidCommitValue);
    }

    if !valid_commit(
        untrusted_sh.commit.clone(),
        untrusted_sh.validators.clone(),
        validator.clone(),
    )
    .eval()
    {
        return Err(Error::ImplementationSpecific);
    }

    if !is_monotonic_bft_time(untrusted_sh.header.clone(), trusted_state.header.clone()).eval() {
        return Err(Error::NonMonotonicBftTime);
    }

    if !is_monotonic_height(trusted_state.header.clone(), untrusted_sh.header.clone()).eval() {
        return Err(Error::NonIncreasingHeight);
    }

    // XXX: why not integrate this into next_validators_match check?
    if !invalid_next_validator_set(
        trusted_state.clone(),
        untrusted_sh.clone(),
        untrusted_next_vals.clone(),
    )
    .eval()
    {
        return Err(Error::InvalidNextValidatorSet);
    }

    if !has_sufficient_validators_overlap(
        untrusted_sh.commit.clone(),
        trusted_state.validators.clone(),
        trust_level.clone(),
        calculator.clone(),
    )
    .eval()
    {
        return Err(Error::InsufficientVotingPower);
    }

    if !has_sufficient_signers_overlap(
        untrusted_sh.commit.clone(),
        untrusted_vals.clone(),
        trust_level.clone(),
        calculator.clone(),
    )
    .eval()
    {
        return Err(Error::InvalidCommit);
    }

    Ok(())
}

fn main() {
    let now = SystemTime::now();

    let trusted_state = TrustedState {
        header: Header {
            height: 9,
            bft_time: now + Duration::from_secs(1),
            validator_set_hash: 34,
            next_validator_set_hash: 35,
            hash: 9,
        },
        validators: ValidatorSet { hash: 34 },
    };

    let untrusted_sh = SignedHeader {
        header: Header {
            height: 10,
            bft_time: now + Duration::from_secs(10),
            validator_set_hash: 99,
            next_validator_set_hash: 100,
            hash: 10,
        },
        commit: Commit { header_hash: 10 },
        validators: ValidatorSet { hash: 100 },
        validator_hash: 100,
    };

    let untrusted_vals = ValidatorSet { hash: 100 };
    let untrusted_next_vals = ValidatorSet { hash: 101 };

    let trust_level = TrustLevel {
        numerator: 1,
        denominator: 3,
    };

    #[derive(Copy, Clone)]
    struct MockCommitValidator;
    impl CommitValidator for MockCommitValidator {
        fn validate(&self, _: &Commit, _: &ValidatorSet) -> Result<(), Error> {
            Ok(())
        }
    }

    #[derive(Copy, Clone)]
    struct MockVotingPowerCalculator;
    impl VotingPowerCalculator for MockVotingPowerCalculator {
        fn voting_power_in(&self, _: &Commit, _: &ValidatorSet) -> Result<u64, Error> {
            Ok(31)
        }

        fn total_power_of(&self, _: &ValidatorSet) -> Result<u64, Error> {
            Ok(42)
        }
    }

    #[derive(Copy, Clone)]
    struct MockHeaderHasher;

    impl HeaderHasher for MockHeaderHasher {
        fn hash(&self, header: &Header) -> Hash {
            header.hash
        }
    }

    let pred = verify_pred(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_level,
        MockCommitValidator,
        MockVotingPowerCalculator,
        MockHeaderHasher,
    );

    #[cfg(feature = "inspect-dot")]
    println!("{}", pred.inspect().to_graph());

    #[cfg(feature = "inspect-text")]
    println!("{}", pred.inspect());
}
