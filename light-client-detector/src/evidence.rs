use std::cmp::Ordering;

use tendermint::{
    block::{signed_header::SignedHeader, Header},
    evidence::{ConflictingBlock, LightClientAttackEvidence},
    validator,
};

use tendermint_light_client::verifier::types::LightBlock;

/// Determines the type of attack and then forms the evidence filling out
/// all the fields such that it is ready to be sent to a full node.
pub fn make_evidence(
    conflicted: LightBlock,
    trusted: LightBlock,
    common: LightBlock,
) -> LightClientAttackEvidence {
    let conflicting_header_is_invalid = conflicting_header_is_invalid(
        &conflicted.signed_header.header,
        &trusted.signed_header.header,
    );

    let conflicting_block = ConflictingBlock {
        signed_header: conflicted.signed_header,
        validator_set: conflicted.validators,
    };

    let byzantine_validators = get_byzantine_validators(
        &conflicting_block,
        &common.validators,
        &trusted.signed_header,
    );

    let witness = if conflicting_header_is_invalid {
        common
    } else {
        trusted
    };

    LightClientAttackEvidence {
        conflicting_block,
        byzantine_validators,
        common_height: witness.height(),
        total_voting_power: witness.validators.total_voting_power().unwrap_or_default(),
        timestamp: witness.time(),
    }
}

/// Take a trusted header and match it againt a conflicting header
/// to determine whether the conflicting header was the product of a valid state transition
/// or not. If it is then all the deterministic fields of the header should be the same.
/// If not, it is an invalid header and constitutes a lunatic attack.
fn conflicting_header_is_invalid(conflicted: &Header, trusted: &Header) -> bool {
    trusted.validators_hash != conflicted.validators_hash
        || trusted.next_validators_hash != conflicted.next_validators_hash
        || trusted.consensus_hash != conflicted.consensus_hash
        || trusted.app_hash != conflicted.app_hash
        || trusted.last_results_hash != conflicted.last_results_hash
}

/// Find out what style of attack `LightClientAttackEvidence` was and then works out who
/// the malicious validators were and returns them. This is used both for forming the `byzantine_validators`
/// field and for validating that it is correct. Validators are ordered based on validator power.
fn get_byzantine_validators(
    conflicted: &ConflictingBlock,
    common_validators: &validator::Set,
    trusted: &SignedHeader,
) -> Vec<validator::Info> {
    // First check if the header is invalid. This means that it is a lunatic attack and therefore we take the
    // validators who are in the `common_validators` and voted for the lunatic header
    if conflicting_header_is_invalid(&conflicted.signed_header.header, &trusted.header) {
        find_lunatic_validators(conflicted, common_validators)
    } else if trusted.commit.round == conflicted.signed_header.commit.round {
        // This is an equivocation attack as both commits are in the same round. We then find the validators
        // from the conflicting light block validator set that voted in both headers.
        // Validator hashes are the same therefore the indexing order of validators are the same and thus we
        // only need a single loop to find the validators that voted twice.

        find_equivocating_validators(conflicted, trusted)
    } else {
        // if the rounds are different then this is an amnesia attack. Unfortunately, given the nature of the attack,
        // we aren't able yet to deduce which are malicious validators and which are not hence we return an
        // empty validator set.

        Vec::new()
    }
}

fn find_lunatic_validators(
    conflicted: &ConflictingBlock,
    common_validators: &validator::Set,
) -> Vec<validator::Info> {
    let mut validators = Vec::new();

    for commit_sig in &conflicted.signed_header.commit.signatures {
        if !commit_sig.is_commit() {
            continue;
        }

        let validator = commit_sig
            .validator_address()
            .and_then(|addr| common_validators.validator(addr));

        if let Some(validator) = validator {
            validators.push(validator);
        }
    }

    validators.sort_by(cmp_voting_power_then_address);
    validators
}

fn find_equivocating_validators(
    conflicted: &ConflictingBlock,
    trusted: &SignedHeader,
) -> Vec<validator::Info> {
    let mut validators = Vec::new();

    for (i, sig_a) in conflicted
        .signed_header
        .commit
        .signatures
        .iter()
        .enumerate()
    {
        if sig_a.is_absent() {
            continue;
        }

        let sig_b = &trusted.commit.signatures[i];
        if sig_b.is_nil() {
            continue;
        }

        let validator = sig_a
            .validator_address()
            .and_then(|addr| conflicted.validator_set.validator(addr));

        if let Some(validator) = validator {
            validators.push(validator);
        }
    }

    validators.sort_by(cmp_voting_power_then_address);
    validators
}

fn cmp_voting_power_then_address(a: &validator::Info, b: &validator::Info) -> Ordering {
    a.power
        .cmp(&b.power)
        .then_with(|| a.address.cmp(&b.address))
}
