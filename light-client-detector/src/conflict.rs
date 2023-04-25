use tendermint::{crypto::Sha256, evidence::LightClientAttackEvidence, merkle::MerkleHash};
use tendermint_light_client::verifier::types::LightBlock;
use tracing::{error, error_span, warn};

use super::{
    error::Error, evidence::make_evidence, examine::examine_conflicting_header_against_trace,
    provider::Provider, trace::Trace,
};

#[derive(Clone, Debug)]
pub struct GatheredEvidence {
    pub witness_trace: Trace,

    pub against_primary: LightClientAttackEvidence,
    pub against_witness: Option<LightClientAttackEvidence>,
}

/// Handles the primary style of attack, which is where a primary and witness have
/// two headers of the same height but with different hashes.
///
/// If a primary provider is available, then we will also attempt to gather evidence against the
/// witness by examining the witness's trace and holding the primary as the source of truth.
pub async fn gather_evidence_from_conflicting_headers<H>(
    primary: Option<&Provider>,
    witness: &Provider,
    primary_trace: &Trace,
    challenging_block: &LightBlock,
) -> Result<GatheredEvidence, Error>
where
    H: Sha256 + MerkleHash + Default,
{
    let _span =
        error_span!("gather_evidence_from_conflicting_headers", witness = %witness.peer_id())
            .entered();

    let (witness_trace, primary_block) =
        examine_conflicting_header_against_trace::<H>(primary_trace, challenging_block, witness)
            .map_err(|e| {
                error!("Error validating witness's divergent header: {e}");

                e // FIXME: Return special error
            })?;

    warn!("ATTEMPTED ATTACK DETECTED. Gathering evidence against primary by witness...");

    // We are suspecting that the primary is faulty, hence we hold the witness as the source of truth
    // and generate evidence against the primary that we can send to the witness

    let common_block = witness_trace.first();
    let trusted_block = witness_trace.last();

    let evidence_against_primary = make_evidence(
        primary_block.clone(),
        trusted_block.clone(),
        common_block.clone(),
    );

    if primary_block.signed_header.commit.round != trusted_block.signed_header.commit.round {
        error!(
            "The light client has detected, and prevented, an attempted amnesia attack.
            We think this attack is pretty unlikely, so if you see it, that's interesting to us.
            Can you let us know by opening an issue through https://github.com/tendermint/tendermint/issues/new"
        );
    }

    let Some(primary) = primary else {
        return Ok(GatheredEvidence {
            witness_trace,
            against_primary: evidence_against_primary,
            against_witness: None,
        });
    };

    // This may not be valid because the witness itself is at fault. So now we reverse it, examining the
    // trace provided by the witness and holding the primary as the source of truth. Note: primary may not
    // respond but this is okay as we will halt anyway.
    let (primary_trace, witness_block) =
        examine_conflicting_header_against_trace(&witness_trace, &primary_block, primary).map_err(
            |e| {
                error!("Error validating primary's divergent header: {e}");

                e // FIXME: Return special error
            },
        )?;

    warn!("Gathering evidence against witness by primary...");

    // We now use the primary trace to create evidence against the witness and send it to the primary
    let common_block = primary_trace.first();
    let trusted_block = primary_trace.last();

    let evidence_against_witness =
        make_evidence(witness_block, trusted_block.clone(), common_block.clone());

    Ok(GatheredEvidence {
        witness_trace,
        against_primary: evidence_against_primary,
        against_witness: Some(evidence_against_witness),
    })
}
