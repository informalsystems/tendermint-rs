use tendermint::{crypto::Sha256, merkle::MerkleHash};
use tendermint_light_client::{
    state::State,
    store::{memory::MemoryStore, LightStore},
    verifier::types::{LightBlock, Status},
};

use super::{error::Error, provider::Provider, trace::Trace};

// examineConflictingHeaderAgainstTrace takes a trace from one provider and a divergent header that
// it has received from another and performs verifySkipping at the heights of each of the intermediate
// headers in the trace until it reaches the divergentHeader. 1 of 2 things can happen.
//
//  1. The light client verifies a header that is different to the intermediate header in the trace. This
//     is the bifurcation point and the light client can create evidence from it
//  2. The source stops responding, doesn't have the block or sends an invalid header in which case we
//     return the error and remove the witness
//
// CONTRACT:
//  1. Trace can not be empty len(trace) > 0
//  2. The last block in the trace can not be of a lower height than the target block
//     trace[len(trace)-1].Height >= targetBlock.Height
//  3. The last block in the trace is conflicting with the target block
pub fn examine_conflicting_header_against_trace<H>(
    trace: &Trace,
    target_block: &LightBlock,
    source: &Provider,
) -> Result<(Trace, LightBlock), Error>
where
    H: Sha256 + MerkleHash + Default,
{
    let trusted_block = trace.first();

    if target_block.height() < trusted_block.height() {
        return Err(Error::target_block_lower_than_trusted(
            target_block.height(),
            trusted_block.height(),
        ));
    };

    let mut previously_verified_block =
        check_trusted_block::<H>(source, trusted_block, target_block)?;

    for trace_block in trace.iter().skip(1) {
        let result = examine_conflicting_header_against_trace_block::<H>(
            source,
            trace_block,
            target_block,
            previously_verified_block,
        )?;

        match result {
            ExaminationResult::Continue(prev_verified_block) => {
                previously_verified_block = prev_verified_block;
                continue;
            },
            ExaminationResult::Divergence(source_trace, trace_block) => {
                return Ok((source_trace, trace_block));
            },
        }
    }

    // We have reached the end of the trace. This should never happen. This can only happen if one of the stated
    // prerequisites to this function were not met.
    // Namely that either trace[len(trace)-1].Height < targetBlock.Height
    // or that trace[i].Hash() != targetBlock.Hash()
    Err(Error::no_divergence())
}

#[derive(Debug)]
pub enum ExaminationResult {
    Continue(LightBlock),
    Divergence(Trace, LightBlock),
}

fn check_trusted_block<H>(
    source: &Provider,
    trusted_block: &LightBlock,
    target_block: &LightBlock,
) -> Result<LightBlock, Error>
where
    H: Sha256 + MerkleHash + Default,
{
    // This case only happens in a forward lunatic attack. We treat the block with the
    // height directly after the targetBlock as the divergent block
    if trusted_block.height() > target_block.height() {
        // sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
        // was correctly verified we should expect monotonically increasing time. This means that if the block at
        // the end of the trace has a lesser time than the target block then all blocks in the trace should have a
        // lesser time
        if trusted_block.time() > target_block.time() {
            return Err(Error::trace_block_after_target_block(
                trusted_block.time(),
                target_block.time(),
            ));
        }
    }

    // get the corresponding block from the source to verify and match up against the traceBlock
    let source_block = if trusted_block.height() == target_block.height() {
        target_block.clone()
    } else {
        source
            .fetch_light_block(trusted_block.height())
            .map_err(Error::light_client)?
    };

    let source_block_hash = source_block.signed_header.header.hash_with::<H>();
    let trace_block_hash = trusted_block.signed_header.header.hash_with::<H>();

    // the first block in the trace MUST be the same to the light block that the source produces
    // else we cannot continue with verification.
    if source_block_hash != trace_block_hash {
        Err(Error::trusted_hash_different_from_source_first_block(
            source_block_hash,
            trace_block_hash,
        ))
    } else {
        Ok(source_block)
    }
}

// check of primary is same as witness block at that height

fn examine_conflicting_header_against_trace_block<H>(
    source: &Provider,
    trace_block: &LightBlock,
    target_block: &LightBlock,
    prev_verified_block: LightBlock,
) -> Result<ExaminationResult, Error>
where
    H: Sha256 + MerkleHash + Default,
{
    // This case only happens in a forward lunatic attack. We treat the block with the
    // height directly after the targetBlock as the divergent block
    if trace_block.height() > target_block.height() {
        // sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
        // was correctly verified we should expect monotonically increasing time. This means that if the block at
        // the end of the trace has a lesser time than the target block then all blocks in the trace should have a
        // lesser time
        if trace_block.time() > target_block.time() {
            return Err(Error::trace_block_after_target_block(
                trace_block.time(),
                target_block.time(),
            ));
        }

        // Before sending back the divergent block and trace we need to ensure we have verified
        // the final gap between the previouslyVerifiedBlock and the targetBlock
        if prev_verified_block.height() != target_block.height() {
            let source_trace = verify_skipping(source, prev_verified_block, target_block.clone())?;

            return Ok(ExaminationResult::Divergence(
                source_trace,
                trace_block.clone(),
            ));
        }
    }

    // get the corresponding block from the source to verify and match up against the traceBlock
    let source_block = if trace_block.height() == target_block.height() {
        target_block.clone()
    } else {
        source
            .fetch_light_block(trace_block.height())
            .map_err(Error::light_client)?
    };

    let source_block_hash = source_block.signed_header.header.hash_with::<H>();
    let trace_block_hash = trace_block.signed_header.header.hash_with::<H>();

    // we check that the source provider can verify a block at the same height of the
    // intermediate height
    let source_trace = verify_skipping(source, prev_verified_block, source_block.clone())?;

    // check if the headers verified by the source has diverged from the trace
    if source_block_hash != trace_block_hash {
        // Bifurcation point found!
        return Ok(ExaminationResult::Divergence(
            source_trace,
            trace_block.clone(),
        ));
    }

    // headers are still the same, continue
    Ok(ExaminationResult::Continue(source_block))
}

fn verify_skipping(
    source: &Provider,
    trusted: LightBlock,
    target: LightBlock,
) -> Result<Trace, Error> {
    let target_height = target.height();

    let mut store = MemoryStore::new();
    store.insert(trusted, Status::Trusted);
    store.insert(target, Status::Unverified);

    let mut state = State::new(store);

    let _ = source
        .verify_to_height_with_state(target_height, &mut state)
        .map_err(Error::light_client)?;

    let blocks = state.get_trace(target_height);
    let source_trace = Trace::new(blocks)?;

    Ok(source_trace)
}
