use std::{thread, time::Duration};

use tracing::{debug, warn};

use tendermint::block::signed_header::SignedHeader;
use tendermint::evidence::LightClientAttackEvidence;
use tendermint_light_client::light_client::TargetOrLatest;
use tendermint_light_client::verifier::errors::ErrorExt;
use tendermint_light_client::verifier::types::LightBlock;

use super::{
    error::Error, gather_evidence_from_conflicting_headers, provider::Provider, trace::Trace,
};

#[derive(Clone, Debug)]
pub struct Divergence {
    pub evidence: LightClientAttackEvidence,
    pub challenging_block: LightBlock,
}

/// Given a primary trace and a witness, detect any divergence between the two,
/// by querying the witness for the same header as the last header in the primary trace
/// (ie. the target block), and comparing the hashes.
///
/// If the hashes match, then no divergence has been detected and the target block can be trusted.
///
/// If the hashes do not match, then the witness has provided a conflicting header.
/// This could possibly imply an attack on the light client.
/// In this case, we need to verify the witness's header using the same skipping verification
/// and then we need to find the point that the headers diverge and examine this for any evidence of an attack.
/// We then attempt to find the bifurcation point and if successful construct the evidence of an
/// attack to report to the witness.
pub async fn detect_divergence(
    witness: &mut Provider,
    primary_trace: Vec<LightBlock>,
    max_clock_drift: Duration,
    max_block_lag: Duration,
) -> Result<Option<Divergence>, Error> {
    let primary_trace = Trace::new(primary_trace)?;

    let last_verified_block = primary_trace.last();
    let last_verified_header = &last_verified_block.signed_header;

    debug!(
        end_block_height = %last_verified_header.header.height,
        end_block_hash = %last_verified_header.header.hash(),
        length = primary_trace.len(),
        "Running detector against primary trace"
    );

    let result = compare_new_header_with_witness(
        last_verified_header,
        witness,
        max_clock_drift,
        max_block_lag,
    );

    match result {
        // No divergence found
        Ok(()) => Ok(None),

        // We have conflicting headers. This could possibly imply an attack on the light client.
        // First we need to verify the witness's header using the same skipping verification and then we
        // need to find the point that the headers diverge and examine this for any evidence of an attack.
        //
        // We combine these actions together, verifying the witnesses headers and outputting the trace
        // which captures the bifurcation point and if successful provides the information to create valid evidence.
        Err(CompareError::ConflictingHeaders(challenging_block)) => {
            warn!(
                witness = %witness.peer_id(),
                height  = %challenging_block.height(),
                "Found conflicting headers between primary and witness"
            );

            // Gather the evidence to report from the conflicting headers
            let evidence = gather_evidence_from_conflicting_headers(
                None,
                witness,
                &primary_trace,
                &challenging_block,
            )
            .await?;

            Ok(Some(Divergence {
                evidence: evidence.against_primary,
                challenging_block: *challenging_block,
            }))
        },

        Err(CompareError::BadWitness) => {
            // These are all melevolent errors and should result in removing the witness
            debug!(witness = %witness.peer_id(), "witness returned an error during header comparison, removing...");

            Err(Error::bad_witness())
        },

        Err(CompareError::Other(e)) => {
            // Benign errors which can be ignored
            debug!(witness = %witness.peer_id(), "error in light block request to witness: {e}");

            Err(Error::light_client(e))
        },
    }
}

#[derive(Debug)]
pub enum CompareError {
    BadWitness,
    Other(tendermint_light_client::errors::Error),
    ConflictingHeaders(Box<LightBlock>),
}

/// compareNewHeaderWithWitness takes the verified header from the primary and compares it with a
/// header from a specified witness. The function can return one of three errors:
///
/// 1: errConflictingHeaders -> there may have been an attack on this light client
/// 2: errBadWitness -> the witness has either not responded, doesn't have the header or has given us an invalid one
///
/// Note: In the case of an invalid header we remove the witness
///
/// 3: nil -> the hashes of the two headers match
pub fn compare_new_header_with_witness(
    new_header: &SignedHeader,
    witness: &mut Provider,
    max_clock_drift: Duration,
    max_block_lag: Duration,
) -> Result<(), CompareError> {
    let light_block = check_against_witness(new_header, witness, max_clock_drift, max_block_lag)?;

    if light_block.signed_header.header.hash() != new_header.header.hash() {
        return Err(CompareError::ConflictingHeaders(Box::new(light_block)));
    }

    Ok(())
}

fn check_against_witness(
    sh: &SignedHeader,
    witness: &mut Provider,
    max_clock_drift: Duration,
    max_block_lag: Duration,
) -> Result<LightBlock, CompareError> {
    let _span =
        tracing::debug_span!("check_against_witness", witness = %witness.peer_id()).entered();

    let light_block = witness.fetch_light_block(sh.header.height);

    match light_block {
        // No error means we move on to checking the hash of the two headers
        Ok(lb) => Ok(lb),

        // The witness hasn't been helpful in comparing headers, we mark the response and continue
        // comparing with the rest of the witnesses
        Err(e) if e.detail().is_io() => {
            debug!("The witness hasn't been helpful in comparing headers");

            Err(CompareError::BadWitness)
        },

        // The witness' head of the blockchain is lower than the height of the primary.
        // This could be one of two things:
        //     1) The witness is lagging behind
        //     2) The primary may be performing a lunatic attack with a height and time in the future
        Err(e) if e.detail().is_height_too_high() => {
            debug!("The witness' head of the blockchain is lower than the height of the primary");

            let light_block = witness
                .get_target_block_or_latest(sh.header.height)
                .map_err(|_| CompareError::BadWitness)?;

            let light_block = match light_block {
                // If the witness caught up and has returned a block of the target height then we can
                // break from this switch case and continue to verify the hashes
                TargetOrLatest::Target(light_block) => return Ok(light_block),

                // Otherwise we continue with the checks
                TargetOrLatest::Latest(light_block) => light_block,
            };

            // The witness' last header is below the primary's header.
            // We check the times to see if the blocks have conflicting times
            debug!("The witness' last header is below the primary's header");

            if !light_block.time().before(sh.header.time) {
                return Err(CompareError::ConflictingHeaders(Box::new(light_block)));
            }

            // The witness is behind. We wait for a period WAITING = 2 * DRIFT + LAG.
            // This should give the witness ample time if it is a participating member
            // of consensus to produce a block that has a time that is after the primary's
            // block time. If not the witness is too far behind and the light client removes it
            let wait_time = 2 * max_clock_drift + max_block_lag;
            debug!("The witness is behind. We wait for {wait_time:?}");

            thread::sleep(wait_time);

            let light_block = witness
                .get_target_block_or_latest(sh.header.height)
                .map_err(|_| CompareError::BadWitness)?;

            let light_block = match light_block {
                // If the witness caught up and has returned a block of the target height then we can
                // return and continue to verify the hashes
                TargetOrLatest::Target(light_block) => return Ok(light_block),

                // Otherwise we continue with the checks
                TargetOrLatest::Latest(light_block) => light_block,
            };

            // The witness still doesn't have a block at the height of the primary.
            // Check if there is a conflicting time
            if !light_block.time().before(sh.header.time) {
                return Err(CompareError::ConflictingHeaders(Box::new(light_block)));
            }

            // Following this request response procedure, the witness has been unable to produce a block
            // that can somehow conflict with the primary's block. We thus conclude that the witness
            // is too far behind and thus we return an error.
            //
            // NOTE: If the clock drift / lag has been miscalibrated it is feasible that the light client has
            // drifted too far ahead for any witness to be able provide a comparable block and thus may allow
            // for a malicious primary to attack it
            Err(CompareError::BadWitness)
        },

        Err(other) => Err(CompareError::Other(other)),
    }
}
