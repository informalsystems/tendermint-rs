use std::cmp::Ordering;

use tendermint::{
    block::{signed_header::SignedHeader, Header},
    evidence::{ConflictingBlock, Evidence, LightClientAttackEvidence},
    validator,
};
use tendermint_light_client_verifier::types::Status;

use crate::{
    evidence::EvidenceReporter,
    state::State,
    store::{memory::MemoryStore, LightStore},
    supervisor::Instance,
    verifier::{operations::Hasher, types::LightBlock},
};

pub use super::error::DetectorError;

pub fn handle_conflicting_headers_and_report_evidence(
    witness: &Instance,
    verified_block: &LightBlock,
    trusted_block: &LightBlock,
    witness_block: &LightBlock,
    hasher: &dyn Hasher,
    evidence_reporter: &dyn EvidenceReporter,
) -> Result<Option<LightClientAttackEvidence>, DetectorError> {
    let evidence = handle_conflicting_headers(
        witness,
        verified_block,
        trusted_block,
        witness_block,
        hasher,
    )?;

    if let Some(evidence) = &evidence {
        tracing::error!(
            ev = ?evidence,
            primary = %"TODO",
            witness = %witness.light_client.peer,
            "ATTEMPTED ATTACK DETECTED. Sending evidence against primary to witness",
        );

        let hash = evidence_reporter
            .report(Evidence::from(evidence.clone()), witness.light_client.peer)
            .map_err(DetectorError::io)?;

        tracing::error!(
            ev = ?evidence,
            primary = %"TODO",
            witness = %witness.light_client.peer,
            hash = %hash,
            "Evidence sent to witness",
        );
    }

    Ok(evidence)
}

/// Handles the primary style of attack, which is where a primary and witness have
/// two headers of the same height but with different hashes
pub fn handle_conflicting_headers(
    witness: &Instance,
    verified_block: &LightBlock,
    trusted_block: &LightBlock,
    witness_block: &LightBlock,
    hasher: &dyn Hasher,
) -> Result<Option<LightClientAttackEvidence>, DetectorError> {
    let (witness_trace, primary_block) = examine_conflicting_header_against_trace(
        trusted_block,
        verified_block,
        witness_block,
        witness,
        hasher,
    )?;

    let common_block = witness_trace.first().unwrap(); // FIXME
    let trusted_block = witness_trace.last().unwrap(); // FIXME

    let evidence_against_primary = make_evidence(
        primary_block.clone(),
        trusted_block.clone(),
        common_block.clone(),
    );

    if primary_block.signed_header.commit.round != trusted_block.signed_header.commit.round {
        tracing::error!(
            "The light client has detected, and prevented, an attempted amnesia attack.
            We think this attack is pretty unlikely, so if you see it, that's interesting to us.
            Can you let us know by opening an issue through https://github.com/tendermint/tendermint/issues/new"
        );
    }

    Ok(Some(evidence_against_primary))
}

enum Examination {
    Continue(LightBlock),
    Bifurcation(Vec<LightBlock>, LightBlock),
}

fn examine_conflicting_header_against_trace_block(
    source: &Instance,
    index: usize,
    trace_block: &LightBlock,
    target_block: &LightBlock,
    prev_verified_block: Option<LightBlock>,
    hasher: &dyn Hasher,
) -> Result<Examination, DetectorError> {
    // This case only happens in a forward lunatic attack. We treat the block with the
    // height directly after the targetBlock as the divergent block
    if trace_block.height() > target_block.height() {
        // sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
        // was correctly verified we should expect monotonically increasing time. This means that if the block at
        // the end of the trace has a lesser time than the target block then all blocks in the trace should have a
        // lesser time
        if trace_block.time() > target_block.time() {
            return Err(DetectorError::trace_block_after_target_block(
                trace_block.time(),
                target_block.time(),
            ));
        }

        // Before sending back the divergent block and trace we need to ensure we have verified
        // the final gap between the previouslyVerifiedBlock and the targetBlock
        if let Some(prev_verified_block) = &prev_verified_block {
            if prev_verified_block.height() != target_block.height() {
                let source_trace =
                    verify_skipping(source, prev_verified_block.clone(), target_block.clone())?;

                return Ok(Examination::Bifurcation(source_trace, trace_block.clone()));
            }
        }
    }

    // get the corresponding block from the source to verify and match up against the traceBlock
    let source_block = if trace_block.height() == target_block.height() {
        target_block.clone()
    } else {
        let mut state = State::new(MemoryStore::new());
        source
            .light_client
            .get_or_fetch_block(trace_block.height(), &mut state)
            .map(|(lb, _)| lb)
            .map_err(DetectorError::light_client)?
    };

    // The first block in the trace MUST be the same to the light block that the source produces
    // else we cannot continue with verification.
    if index == 0 {
        if hasher.hash_header(&source_block.signed_header.header)
            != hasher.hash_header(&trace_block.signed_header.header)
        {
            return Err(
                DetectorError::trusted_hash_different_from_source_first_block(
                    hasher.hash_header(&source_block.signed_header.header),
                    hasher.hash_header(&trace_block.signed_header.header),
                ),
            );
        }

        return Ok(Examination::Continue(source_block));
    }

    // we check that the source provider can verify a block at the same height of the
    // intermediate height
    let source_trace = verify_skipping(
        source,
        prev_verified_block.unwrap(), // FIXME
        source_block.clone(),
    )?;

    // check if the headers verified by the source has diverged from the trace
    if hasher.hash_header(&source_block.signed_header.header)
        != hasher.hash_header(&trace_block.signed_header.header)
    {
        // Bifurcation point found!
        return Ok(Examination::Bifurcation(source_trace, trace_block.clone()));
    }

    // headers are still the same, continue
    Ok(Examination::Continue(source_block))
}

fn examine_conflicting_header_against_trace(
    trusted_block: &LightBlock,
    verified_block: &LightBlock,
    target_block: &LightBlock,
    source: &Instance,
    hasher: &dyn Hasher,
) -> Result<(Vec<LightBlock>, LightBlock), DetectorError> {
    if target_block.height() < trusted_block.height() {
        return Err(DetectorError::target_block_lower_than_trusted(
            target_block.height(),
            trusted_block.height(),
        ));
    }

    let mut previously_verified_block: Option<LightBlock> = None;

    for (index, &trace_block) in [trusted_block, verified_block].iter().enumerate() {
        let result = examine_conflicting_header_against_trace_block(
            source,
            index,
            trace_block,
            target_block,
            previously_verified_block,
            hasher,
        )?;

        match result {
            Examination::Continue(prev_verified_block) => {
                previously_verified_block = Some(prev_verified_block);
                continue;
            },
            Examination::Bifurcation(source_trace, trace_block) => {
                return Ok((source_trace, trace_block));
            },
        }
    }

    // We have reached the end of the trace. This should never happen. This can only happen if one of the stated
    // prerequisites to this function were not met.
    // Namely that either trace[len(trace)-1].Height < targetBlock.Height or that trace[i].Hash() != targetBlock.Hash()
    Err(DetectorError::no_divergence())
}

/// Determines the type of attack and then forms the evidence filling out
/// all the fields such that it is ready to be sent to a full node.
fn make_evidence(
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
        total_voting_power: witness.validators.total_voting_power(),
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

fn verify_skipping(
    source: &Instance,
    trusted: LightBlock,
    target: LightBlock,
) -> Result<Vec<LightBlock>, DetectorError> {
    let target_height = target.height();

    let mut store = MemoryStore::new();
    store.insert(trusted, Status::Trusted);
    store.insert(target, Status::Unverified);

    let mut state = State::new(store);

    let _ = source
        .light_client
        .verify_to_target(target_height, &mut state)
        .map_err(DetectorError::light_client)?;

    let source_trace = state.get_trace(target_height);

    Ok(source_trace)
}
