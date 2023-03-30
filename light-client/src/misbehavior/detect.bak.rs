//pub async fn detect_divergence(
//    primary: &mut Provider,
//    witnesses: &mut [Provider],
//    primary_trace: Vec<LightBlock>,
//    max_clock_drift: Duration,
//    max_block_lag: Duration,
//    now: Time,
//) -> Result<(), DivergenceError> {
//    let primary_trace =
//        Trace::new(primary_trace).map_err(|e| DivergenceError::trace_too_short(e.trace))?;

//    if witnesses.is_empty() {
//        return Err(DivergenceError::no_witnesses());
//    }

//    let last_verified_block = primary_trace.last();
//    let last_verified_header = &last_verified_block.signed_header;

//    debug!(
//        end_block_height = %last_verified_header.header.height,
//        end_block_hash = %last_verified_header.header.hash(),
//        length = primary_trace.len(),
//        "Running detector against primary trace"
//    );

//    let mut header_matched = false;
//    let mut witnesses_to_remove = Vec::new();

//    for mut witness in witnesses {
//        let result = compare_new_header_with_witness(
//            last_verified_header,
//            witness,
//            max_clock_drift,
//            max_block_lag,
//        );

//        match result {
//            // At least one header matched
//            Ok(()) => {
//                header_matched = true;
//            },

//            // We have conflicting headers. This could possibly imply an attack on the light client.
//            // First we need to verify the witness's header using the same skipping verification and then we
//            // need to find the point that the headers diverge and examine this for any evidence of an attack.
//            //
//            // We combine these actions together, verifying the witnesses headers and outputting the trace
//            // which captures the bifurcation point and if successful provides the information to create valid evidence.
//            Err(CompareError::ConflictingHeaders(challenging_block)) => {
//                warn!(
//                    primary = %primary.peer_id(),
//                    witness = %witness.peer_id(),
//                    height  = %challenging_block.height(),
//                    "Found conflicting headers between primary and witness"
//                );

//                // Gather the evidence to report from the conflicting headers
//                let evidence = gather_evidence_from_conflicting_headers(
//                    Some(primary),
//                    witness,
//                    &primary_trace,
//                    &challenging_block,
//                )
//                .await;

//                match evidence {
//                    Ok(evidence) => {
//                        info!(
//                            primary = %primary.peer_id(),
//                            witness = %witness.peer_id(),
//                            "Found evidence of misbehavior against primary and witness"
//                        );

//                        report_evidence(primary, witness, evidence).await;

//                        info!(
//                            primary = %primary.peer_id(),
//                            witness = %witness.peer_id(),
//                            "Generated evidence and reported it to both primary and witness"
//                        );

//                        return Err(DivergenceError::divergence(evidence));
//                    },

//                    Err(e) => {
//                        error!("Failed to handle conflicting headers: {e}");

//                        // If attempt to generate conflicting headers failed then remove witness
//                        witnesses_to_remove.push(witness.peer_id());
//                    },
//                }
//            },

//            Err(CompareError::BadWitness) => {
//                // These are all melevolent errors and should result in removing the witness
//                debug!(witness = %witness.peer_id(), "witness returned an error during header comparison, removing...");

//                witnesses_to_remove.push(witness.peer_id());
//            },

//            Err(CompareError::Other(e)) => {
//                // Benign errors which can be ignored
//                debug!(witness = %witness.peer_id(), "error in light block request to witness: {e}");
//            },
//        }
//    }

//    // TODO: Report back the witnesses to remove

//    if header_matched {
//        // 1. If we had at least one witness that returned the same header then we
//        // conclude that we can trust the header
//        Ok(())
//    } else {
//        // 2. Else all witnesses have either not responded, don't have the block or sent invalid blocks.
//        Err(DivergenceError::failed_header_cross_referencing())
//    }
//}

//pub async fn report_evidence(primary: &Provider, witness: &Provider, evidence: GatheredEvidence) {
//    debug!(
//        "Evidence against primary: {}",
//        serde_json::to_string_pretty(&evidence.against_primary).unwrap()
//    );

//    let result = witness
//        .report_evidence(Evidence::from(evidence.against_primary))
//        .await;

//    match result {
//        Ok(hash) => {
//            info!("Successfully submitted evidence against primary. Evidence hash: {hash}",);
//        },
//        Err(e) => {
//            error!("Failed to submit evidence against primary: {e}");
//        },
//    }

//    let Some(evidence_against_witness) = evidence.against_witness else {
//        return;
//    };

//    debug!(
//        "Evidence against witness: {}",
//        serde_json::to_string_pretty(&evidence_against_witness).unwrap()
//    );

//    let result = primary
//        .report_evidence(Evidence::from(evidence_against_witness))
//        .await;

//    match result {
//        Ok(hash) => {
//            info!("Successfully submitted evidence against witness. Evidence hash: {hash}",);
//        },
//        Err(e) => {
//            error!("Failed to submit evidence against witness: {e}");
//        },
//    }
//}
