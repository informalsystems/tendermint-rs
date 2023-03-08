use tendermint::{crypto::default::Sha256, evidence::LightClientAttackEvidence};
use tendermint_light_client_verifier::types::Status;

use crate::{
    instance::Instance,
    state::State,
    store::{memory::MemoryStore, LightStore},
    verifier::types::LightBlock,
};

use super::error::DetectorError;
use super::evidence::make_evidence;

// FIXME: Allow the hasher to be configured
type Hasher = Sha256;

/// Handles the primary style of attack, which is where a primary and witness have
/// two headers of the same height but with different hashes
pub fn handle_conflicting_headers(
    witness: &Instance,
    verified_block: &LightBlock,
    trusted_block: &LightBlock,
    witness_block: &LightBlock,
) -> Result<Option<LightClientAttackEvidence>, DetectorError> {
    let (witness_trace, primary_block) = examine_conflicting_header_against_trace(
        trusted_block,
        verified_block,
        witness_block,
        witness,
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
        if source_block.signed_header.header.hash_with::<Hasher>()
            != trace_block.signed_header.header.hash_with::<Hasher>()
        {
            return Err(
                DetectorError::trusted_hash_different_from_source_first_block(
                    source_block.signed_header.header.hash_with::<Hasher>(),
                    trace_block.signed_header.header.hash_with::<Hasher>(),
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
    if source_block.signed_header.header.hash_with::<Hasher>()
        != trace_block.signed_header.header.hash_with::<Hasher>()
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
