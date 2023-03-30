#![allow(unused)]

use std::{borrow::Borrow, thread, time::Duration};

use tendermint::{
    block::signed_header::{self, SignedHeader},
    Time,
};
use tendermint_light_client_verifier::types::LightBlock;
use tracing::{debug, error, info, warn};

use super::{
    conflict::ReportedEvidence,
    error::detector::Error as DetectorError,
    error::divergence::Error as DivergenceError,
    gather_evidence_from_conflicting_headers,
    provider::Provider,
    trace::{Trace, TraceTooShort},
};

pub async fn detect_divergence(
    primary: &mut Provider,
    witnesses: &mut [Provider],
    primary_trace: Vec<LightBlock>,
    max_clock_drift: Duration,
    max_block_lag: Duration,
    now: Time,
) -> Result<(), DivergenceError> {
    let primary_trace =
        Trace::new(primary_trace).map_err(|e| DivergenceError::trace_too_short(e.trace))?;

    if witnesses.is_empty() {
        return Err(DivergenceError::no_witnesses());
    }

    let last_verified_block = primary_trace.last();
    let last_verified_header = &last_verified_block.signed_header;

    debug!(
        end_block_height = %last_verified_header.header.height,
        end_block_hash = %last_verified_header.header.hash(),
        length = primary_trace.len(),
        "Running detector against primary trace"
    );

    let mut header_matched = false;
    let mut witnesses_to_remove = Vec::new();

    for mut witness in witnesses {
        let result = compare_new_header_with_witness(
            last_verified_header,
            witness,
            max_clock_drift,
            max_block_lag,
        );

        match result {
            // At least one header matched
            Ok(()) => {
                header_matched = true;
            },

            // We have conflicting headers. This could possibly imply an attack on the light client.
            // First we need to verify the witness's header using the same skipping verification and then we
            // need to find the point that the headers diverge and examine this for any evidence of an attack.
            //
            // We combine these actions together, verifying the witnesses headers and outputting the trace
            // which captures the bifurcation point and if successful provides the information to create valid evidence.
            Err(CompareError::ConflictingHeaders(challenging_block)) => {
                warn!(
                    primary = %primary.peer_id(),
                    witness = %witness.peer_id(),
                    height  = %challenging_block.height(),
                    "Found conflicting headers between primary and witness"
                );

                // Handle the conflicting headers, generate evidence and report it to both the primary and witness
                let result = gather_evidence_from_conflicting_headers(
                    primary,
                    witness,
                    &primary_trace,
                    &challenging_block,
                )
                .await;

                match result {
                    Ok(reported_evidence) => {
                        info!(
                            primary = %primary.peer_id(),
                            witness = %witness.peer_id(),
                            "Generated evidence"
                        );

                        return Err(DivergenceError::divergence(
                            reported_evidence,
                            challenging_block,
                        ));
                    },

                    Err(e) => {
                        error!("Failed to handle conflicting headers: {e}");

                        // If attempt to generate conflicting headers failed then remove witness
                        witnesses_to_remove.push(witness.peer_id());
                    },
                }
            },

            Err(CompareError::BadWitness) => {
                // These are all melevolent errors and should result in removing the witness
                debug!(witness = %witness.peer_id(), "witness returned an error during header comparison, removing...");

                witnesses_to_remove.push(witness.peer_id());
            },

            Err(CompareError::Other(e)) => {
                // Benign errors which can be ignored
                debug!(witness = %witness.peer_id(), "error in light block request to witness: {e}");
            },
        }
    }

    // TODO: Report back the witnesses to remove

    if header_matched {
        // 1. If we had at least one witness that returned the same header then we
        // conclude that we can trust the header
        Ok(())
    } else {
        // 2. Else all witnesses have either not responded, don't have the block or sent invalid blocks.
        Err(DivergenceError::failed_header_cross_referencing())
    }
}

#[derive(Debug)]
pub enum CompareError {
    BadWitness,
    Other(crate::errors::ErrorDetail),
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
fn compare_new_header_with_witness(
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
    use crate::errors::ErrorDetail;
    use crate::light_client::TargetOrLatest;

    let _span =
        tracing::debug_span!("check_against_witness", witness = %witness.peer_id()).entered();

    let light_block = witness
        .fetch_light_block(sh.header.height)
        .map_err(|e| e.into_detail());

    match light_block {
        // No error means we move on to checking the hash of the two headers
        Ok(lb) => Ok(lb),

        // The witness hasn't been helpful in comparing headers, we mark the response and continue
        // comparing with the rest of the witnesses
        Err(ErrorDetail::Io(_)) => {
            debug!("The witness hasn't been helpful in comparing headers");

            Err(CompareError::BadWitness)
        },

        // The witness' head of the blockchain is lower than the height of the primary.
        // This could be one of two things:
        //     1) The witness is lagging behind
        //     2) The primary may be performing a lunatic attack with a height and time in the future
        Err(ErrorDetail::HeightTooHigh(_)) => {
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
