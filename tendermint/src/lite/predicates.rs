/// Sketch out the pseudo code for a light client
/// That integrates the learning from the last iteration.
/// What we want:
/// + Simple light client specific types, no crypto
/// + Crypto can abstracted into traits which implement crypto specific functions
/// + Express the core verification logic as a composition of predicates to allow mocking
use crate::Hash;


// Some Simplified type which only have the fields needed for core verification
type Height = u64;
struct Header {
    height: Height,
    bft_time: systemTime,
    validator_set_hash: Hash,
    next_validator_set_hash: Hash,
    hash: Hash, // What if we don't have this
}

struct ValidatorSet {
    hash: Hash,
}

struct Commit {
    header_hash: Hash,
}

struct TrustLevel {
    numerator: u64,
    denominator: u64,
}

// Crypto function traits allowing mocking out during testing
trait VotingPowerCalculator: Sized {
    // What kind of errors should we be reporting here?
    fn voting_power_in(commit: &Commit, validators: &Set) -> Result<u64, Error>;
}

trait CommitValidator: Sized {
    fn validate(commit: &Commit, validators: ValidatorSet) -> Result<(), Error>;
}

trait HeaderHasher: Sized {
    fn hash(header: Header) -> Hash; // Or Error?
}

/// Predicates

fn validator_sets_match(signed_header: &SignedHeader, validators: &ValidatorSet) -> bool {
    return signed_header.validator_hash == validators.hash
}

fn next_validators_match(signed_header: &SignedHeader, validators: &ValidatorSet) -> bool {
    return signed_header.validator_hash == validators.hash
}

fn header_matches_commit(
    header: &Header,
    commit: &Commit,
    header_hasher: HeaderHasher) -> bool {
    return header_hasher.hash(header.hash) == commit.header_hash
}

fn valid_commit(commit: &Commit, validator: &CommitValidator) -> bool {
    return validator.validate(commit).is_ok();
}

fn is_within_trusted_period(header: &Header, trusting_period: &Duration, now: &SystemTime) -> bool {
    let header_time: SystemTime = last_header.bft_time.into();
    let expires_at = header_time.add(trusting_period);

    return header_time < now && expired_at > now
}

fn is_monotonic_bft_time(header_a: &Header, header_b: &Header) -> bool {
    return header_b.bft_time >= header_a.bft_time
}

fn is_monotonic_height(header_a: &Header, header_b: &Header) -> bool {
    return header_b.height > header_b.height
}

fn has_sufficient_voting_power(
    commit: &Commit,
    validators: &Validators,
    calculator: &VotingPowerCalculator,
    trust_level: &TrustingLevel) -> bool {

    let voting_power = calculator.voting_power_in(commit, validator_set);

    // XXX: Maybe trust_level doesn't need a very sophisticated type
    return voting_power > trusted_level;
}

fn has_sufficient_validators_overlap(
    untrusted_commit: &Commit,
    trusted_validators: &ValidatorSet,
    trust_level: &TrustLevel,
    calculator: &VotingPowerCalculator) -> bool {

    return has_sufficient_voting_power(untrusted_commit, trusted_validators, trust_level, calculator);
}

fn has_sufficient_signers_overlap(
    untrusted_commit: &Commit,
    untrusted_validators: &ValidatorSet,
    trust_level: &TrustLevel,
    calculator: &VotingPowerCalculator) -> bool {

    return has_sufficient_voting_power(untrusted_commit, untrusted_validators, trust_level, calculator);
}

// TODO: Parameterize by predicates instead of raw types
fn verify(
    trusted_state: &TrustedState,
    untrusted_sh: &SignedHeader,
    untrusted_vals: &ValidatorSet,
    untrusted_next_vals: &ValidatorSet,
    trust_level: &TrustLevel) -> Result<(), Error> { // shouldn't this return a new TrustedState?

    if !validator_sets_match(untrusted_sh.validators, untrusted_vals) {
        return InvalidValidatorSet
    }

    if !next_validators_match(unrtusted_sh, untrusted_next_vals) {
        return InvalidNextValidatorSet
    }

    if !header_matches_commit(untrusted_sh) {
        return InvalidCommitValue
    }

    if !valid_commit(untrusted_sh.commit) {
        return ImplementationSpecific
    }

    if !is_monotonic_bft_time(untusted_sh, trusted_state.header) {
        return NonIncreasingHeight
    }

    if !is_monotonic_height(trusted_state.header, untrusted_sh.header) {
        return NonIncreasingHeight
    }

    // XXX: why not integrate this into next_validators_match check?
    if untrusted_sh.header.height == trusted_state.header.height &&
        trusted_state.validators.hash != untrusted_next_vals.hash {
            return InvalidNextValidatorSet
    }

    if !has_sufficient_validators_overlap(untrusted_sh.commit, trusted_state.validators, trust_level) {
        return InsufficientVotingPower
    }

    if !has_sufficient_signers_overlap(untrusted_sh.commit, untrusted_vals) {
        return InvalidCommit
    }

    return Ok(())
}

//  TODO: Now do the bisection logic as a sequence of verify applications
