//! Fork detection data structures and implementation.

use std::cmp::Ordering;

use tendermint::{
    block::{signed_header::SignedHeader, Header},
    evidence::{ConflictingBlock, LightClientAttackEvidence},
    validator, Time,
};
use tendermint_light_client_verifier::types::{PeerId, Status};

use crate::{
    errors::{Error, ErrorDetail},
    light_client::TargetOrLatest,
    state::State,
    store::{memory::MemoryStore, LightStore},
    supervisor::Instance,
    verifier::{
        errors::ErrorExt,
        operations::{Hasher, ProdHasher},
        types::LightBlock,
    },
};

/// Result of fork detection
#[derive(Debug)]
pub enum ForkDetection {
    /// One or more forks have been detected
    Detected(Vec<Fork>),
    /// No fork has been detected
    NotDetected,
}

/// Types of fork
#[derive(Debug)]
// To be fixed in 0.24
#[allow(clippy::large_enum_variant)]
pub enum Fork {
    /// An actual fork was found for this `LightBlock`
    Forked {
        /// Light block fetched from the primary
        primary: LightBlock,
        /// Light block fetched from a witness
        witness: LightBlock,
    },

    /// The node has been deemed faulty for this `LightBlock`
    Faulty(LightBlock, ErrorDetail),

    /// The node has timed out
    Timeout(LightBlock, ErrorDetail),
}

/// Interface for a fork detector
pub trait ForkDetector: Send + Sync {
    /// Detect forks using the given verified block, trusted block,
    /// and list of witnesses to verify the given light block against.
    fn detect_forks(
        &self,
        verified_block: &LightBlock,
        trusted_block: &LightBlock,
        primary_trace: Vec<LightBlock>,
        primary: &Instance,
        witnesses: &[&Instance],
        now: Time,
    ) -> Result<ForkDetection, Error>;
}

/// A production-ready fork detector which compares
/// light blocks fetched from the witnesses by hash.
/// If the hashes don't match, this fork detector
/// then attempts to verify the light block pulled from
/// the witness against a light block containing only
/// the given trusted state, and then:
///
/// - If the verification succeeds, we have a real fork
/// - If verification fails because of lack of trust, we have a potential fork.
/// - If verification fails for any other reason, the witness is deemed faulty.
pub struct ProdForkDetector {
    hasher: Box<dyn Hasher>,
}

impl ProdForkDetector {
    /// Construct a new fork detector that will use the given header hasher.
    pub fn new(hasher: impl Hasher + 'static) -> Self {
        Self {
            hasher: Box::new(hasher),
        }
    }
}

impl Default for ProdForkDetector {
    fn default() -> Self {
        Self::new(ProdHasher)
    }
}

impl ForkDetector for ProdForkDetector {
    /// Perform fork detection. See the documentation `ProdForkDetector` for details.
    fn detect_forks(
        &self,
        verified_block: &LightBlock,
        trusted_block: &LightBlock,
        primary_trace: Vec<LightBlock>,
        primary: &Instance,
        witnesses: &[&Instance],
        now: Time,
    ) -> Result<ForkDetection, Error> {
        detect_forks(
            verified_block,
            trusted_block,
            primary_trace,
            primary,
            witnesses,
            now,
            self.hasher.as_ref(),
        )
    }
}

/// Perform fork detection. See the documentation `ProdForkDetector` for details.
fn detect_forks(
    verified_block: &LightBlock,
    trusted_block: &LightBlock,
    primary_trace: Vec<LightBlock>,
    primary: &Instance,
    witnesses: &[&Instance],
    now: Time,
    hasher: &dyn Hasher,
) -> Result<ForkDetection, Error> {
    let primary_hash = hasher.hash_header(&verified_block.signed_header.header);

    let mut forks = Vec::with_capacity(witnesses.len());

    // TODO: Do this in parallel
    for witness in witnesses {
        let mut state = State::new(MemoryStore::new());

        let (witness_block, _) = witness
            .light_client
            .get_or_fetch_block(verified_block.height(), &mut state)?;

        let witness_hash = hasher.hash_header(&witness_block.signed_header.header);

        if primary_hash == witness_hash {
            // Hashes match, continue with next witness, if any.
            continue;
        }

        let fault =
            compare_new_header_with_witness(&verified_block.signed_header, witness, hasher)?;

        match fault {
            // At least one header matched
            Fault::None => (),

            // We have conflicting headers. This could possibly imply an attack on the light client.
            // First we need to verify the witness's header using the same skipping verification and then we
            // need to find the point that the headers diverge and examine this for any evidence of an attack.
            //
            // We combine these actions together, verifying the witnesses headers and outputting the trace
            // which captures the bifurcation point and if successful provides the information to create valid evidence.
            Fault::ConflictingHeaders(witness_block, _) => {
                let fork = handle_conflicting_headers(
                    primary,
                    witness,
                    // trusted_block,
                    // verified_block,
                    witness_block,
                    &primary_trace,
                    now,
                    hasher,
                )?;

                // TODO: Record witness to remove

                forks.push(fork);
            },

            // These are all melevolent errors and should result in removing the witness
            Fault::BadWitness(_) => {
                // TODO: Record witness to remove
            },

            // Benign errors which can be ignored unless there was a context canceled
            Fault::Other(_e, _) => {
                // TODO: Print out debug error
            },
        }
    }

    if forks.is_empty() {
        Ok(ForkDetection::NotDetected)
    } else {
        Ok(ForkDetection::Detected(forks))
    }
}

#[allow(clippy::large_enum_variant)]
enum Fault {
    ConflictingHeaders(LightBlock, PeerId),
    BadWitness(PeerId),
    Other(Error, PeerId),
    None,
}

/// Takes the verified header from the primary and compares it with a
/// header from a specified witness. The function can return one of two faults:
///
/// 1: ConflictingHeaders -> there may have been an attack on this light client
/// 2: BadWitness -> the witness has either not responded, doesn't have the header or has given us an invalid one
///
/// Note: In the case of an invalid header we remove the witness
///
/// Otherwise, the hashes of the two headers match
fn compare_new_header_with_witness(
    verified_header: &SignedHeader,
    witness: &Instance,
    hasher: &dyn Hasher,
) -> Result<Fault, Error> {
    let height = verified_header.header.height;

    let light_block = witness
        .light_client
        .get_or_fetch_block(height, &mut State::new(MemoryStore::new()));

    match light_block {
        Ok((light_block, _)) => {
            let header_hash = hasher.hash_header(&verified_header.header);
            let block_hash = hasher.hash_header(&light_block.signed_header.header);

            if header_hash != block_hash {
                Ok(Fault::ConflictingHeaders(
                    light_block,
                    witness.light_client.peer,
                ))
            } else {
                Ok(Fault::None)
            }
        },

        Err(e) if e.detail().is_height_too_high() => {
            let light_block = witness.light_client.get_target_block_or_latest(height)?;

            match light_block {
                // Ff the witness caught up and has returned a block of the target height then we can
                // break from this switch case and continue to verify the hashes
                TargetOrLatest::Target(_) => Ok(Fault::None),

                // The witness' last header is below the primary's header.
                // We check the times to see if the blocks have conflicting times
                TargetOrLatest::Latest(light_block) => {
                    if light_block.time() < verified_header.header.time {
                        Ok(Fault::ConflictingHeaders(
                            light_block,
                            witness.light_client.peer,
                        ))
                    } else {
                        // TODO: Wait for 2 * DRIFT + LAG and try again
                        Ok(Fault::BadWitness(witness.light_client.peer))
                    }
                },
            }
        },

        Err(e) => Ok(Fault::Other(e, witness.light_client.peer)),
    }
}

/// Handles the primary style of attack, which is where a primary and witness have
/// two headers of the same height but with different hashes
fn handle_conflicting_headers(
    primary: &Instance,
    witness: &Instance,
    // verified_block: &LightBlock,
    // trusted_block: &LightBlock,
    witness_block: LightBlock,
    primary_trace: &[LightBlock],
    now: Time,
    hasher: &dyn Hasher,
) -> Result<Fork, Error> {
    let (witness_trace, primary_block) = examine_conflicting_header_against_trace(
        primary_trace,
        witness_block,
        witness,
        now,
        hasher,
    )?;

    let common_block = witness_trace.first().unwrap();
    let trusted_block = witness_trace.last().unwrap(); // FIXME

    let _evidence_against_primary = make_evidence(
        primary_block.clone(),
        trusted_block.clone(),
        common_block.clone(),
    );

    // tracing::error!(
    //     ev = ?evidence_against_primary
    //     primary = %"TODO",
    //     witness = %witness.light_client.peer,
    //     "ATTEMPTED ATTACK DETECTED. Sending evidence againt primary by witness",
    // );

    // TODO: Send evidence to primary

    if primary_block.signed_header.commit.round != trusted_block.signed_header.commit.round {
        // tracing::error!(
        //     "The light client has detected, and prevented, an attempted amnesia attack.
        //     We think this attack is pretty unlikely, so if you see it, that's interesting to us.
        //     Can you let us know by opening an issue through https://github.com/tendermint/tendermint/issues/new"
        // );
    }

    // This may not be valid because the witness itself is at fault. So now we reverse it, examining the
    // trace provided by the witness and holding the primary as the source of truth. Note: primary may not
    // respond but this is okay as we will halt anyway.
    let (primary_trace, witness_block) = examine_conflicting_header_against_trace(
        &witness_trace,
        primary_block,
        primary,
        now,
        hasher,
    )?;

    let (common_block, trusted_block) = (
        primary_trace.first().unwrap(),
        primary_trace.last().unwrap(),
    );

    let _evidence_against_witness =
        make_evidence(witness_block, trusted_block.clone(), common_block.clone());

    // tracing::error!(
    //      ev = ?evidence_against_primary
    //      primary = %"TODO",
    //      witness = %witness.light_client.peer,
    //     "Sending evidence against witness by primary"
    // );

    // TODO: Send evidence to primary

    todo!()
}

fn examine_conflicting_header_against_trace(
    trace: &[LightBlock],
    target_block: LightBlock,
    source: &Instance,
    now: Time,
    hasher: &dyn Hasher,
) -> Result<(Vec<LightBlock>, LightBlock), Error> {
    assert!(!trace.is_empty());

    if target_block.height() < trace[0].height() {
        unreachable!()
        // return Err(Error::other(
        //     format!(
        //         "target block has height lower than the trusted height",
        //         target_block.height(),
        //         trace[0].height()
        //     ),
        // ));
    }

    let mut previously_verified_block: Option<LightBlock> = None;

    for (i, trace_block) in trace.iter().enumerate() {
        // this case only happens in a forward lunatic attack. We treat the block with the
        // height directly after the targetBlock as the divergent block
        if trace_block.height() > target_block.height() {
            // sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
            // was correctly verified we should expect monotonically increasing time. This means that if the block at
            // the end of the trace has a lesser time than the target block then all blocks in the trace should have a
            // lesser time
            if trace_block.time() > target_block.time() {
                unreachable!()
                // return Err(Error::other("sanity check failed: expected trace block to have a lesser time than the target block"));
            }

            // before sending back the divergent block and trace we need to ensure we have verified
            // the final gap between the previouslyVerifiedBlock and the targetBlock
            if let Some(prev_verified_block) = &previously_verified_block {
                if prev_verified_block.height() != target_block.height() {
                    let source_trace =
                        verify_skipping(source, prev_verified_block.clone(), target_block)?;

                    return Ok((source_trace, trace_block.clone()));
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
                .map(|(lb, _)| lb)?
        };

        // The first block in the trace MUST be the same to the light block that the source produces
        // else we cannot continue with verification.
        if i == 0 {
            if hasher.hash_header(&source_block.signed_header.header)
                != hasher.hash_header(&trace_block.signed_header.header)
            {
                unreachable!()
                // return Err(Error::other(
                //     format!(
                //         "trusted block is different to the source's first block. Expected hash: {}, got: {}",
                //         hasher.hash_header(&source_block.signed_header.header),
                //         hasher.hash_header(&trace_block.signed_header.header)
                //     )
                // ));
            }

            previously_verified_block = Some(source_block);
            continue;
        }

        // we check that the source provider can verify a block at the same height of the
        // intermediate height
        {
            let mut store = MemoryStore::new();
            store.insert(previously_verified_block.clone().unwrap(), Status::Trusted);
            store.insert(source_block.clone(), Status::Unverified);
            let mut state = State::new(store);

            let _ = source
                .light_client
                .verify_to_target(source_block.height(), &mut state)?;

            let source_trace = state.get_trace(source_block.height());

            // check if the headers verified by the source has diverged from the trace
            if hasher.hash_header(&source_block.signed_header.header)
                != hasher.hash_header(&trace_block.signed_header.header)
            {
                // Bifurcation point found!
                return Ok((source_trace, trace_block.clone()));
            }

            // headers are still the same. update the previouslyVerifiedBlock
            previously_verified_block = Some(source_block);
        };
    }

    Err(todo!())
}

fn verify_skipping(
    source: &Instance,
    trusted: LightBlock,
    target: LightBlock,
) -> Result<Vec<LightBlock>, Error> {
    let target_height = target.height();

    let mut store = MemoryStore::new();
    store.insert(trusted, Status::Trusted);
    store.insert(target, Status::Unverified);

    let mut state = State::new(store);

    let _ = source
        .light_client
        .verify_to_target(target_height, &mut state)?;

    let source_trace = state.get_trace(target_height);
    Ok(source_trace)
}

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
    let mut validators = Vec::new();

    // First check if the header is invalid. This means that it is a lunatic attack and therefore we take the
    // validators who are in the `common_validators` and voted for the lunatic header
    if conflicting_header_is_invalid(&conflicted.signed_header.header, &trusted.header) {
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
    } else if trusted.commit.round == conflicted.signed_header.commit.round {
        // This is an equivocation attack as both commits are in the same round. We then find the validators
        // from the conflicting light block validator set that voted in both headers.
        // Validator hashes are the same therefore the indexing order of validators are the same and thus we
        // only need a single loop to find the validators that voted twice.

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
    } else {
        // if the rounds are different then this is an amnesia attack. Unfortunately, given the nature of the attack,
        // we aren't able yet to deduce which are malicious validators and which are not hence we return an
        // empty validator set.

        Vec::new()
    }
}

fn cmp_voting_power_then_address(a: &validator::Info, b: &validator::Info) -> Ordering {
    a.power
        .cmp(&b.power)
        .then_with(|| a.address.cmp(&b.address))
}
