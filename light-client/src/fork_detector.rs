//! Fork detection data structures and implementation.



use serde::{Deserialize, Serialize};

use crate::{
    errors::{Error, ErrorKind},
    operations::{Hasher, ProdHasher},
    state::State,
    store::memory::MemoryStore,
    supervisor::Instance,
    types::{LightBlock, PeerId},
};

/// Result of fork detection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    /// One or more forks have been detected
    Detected(Vec<Fork>),
    /// No fork has been detected
    NotDetected,
}

/// Types of fork
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Fork {
    /// An actual fork was found for this `LightBlock`
    Forked {
        /// Light block fetched from the primary
        primary: LightBlock,
        /// Light block fetched from a witness
        witness: LightBlock,
    },
    /// The node has been deemed faulty for this `LightBlock`
    Faulty(LightBlock, ErrorKind),
    /// The node has timed out
    Timeout(PeerId, ErrorKind),
}

/// Interface for a fork detector
pub trait ForkDetector: Send + Sync {
    /// Detect forks using the given verified block, trusted block,
    /// and list of witnesses to verify the given light block against.
    fn detect_forks(
        &self,
        verified_block: &LightBlock,
        //trusted_block: &LightBlock,
        primary: &Instance,
        witnesses: Vec<&Instance>,
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
     //   trusted_block: &LightBlock,
        primary: &Instance,
        witnesses: Vec<&Instance>,
    ) -> Result<ForkDetection, Error> {
        let primary_hash = self
            .hasher
            .hash_header(&verified_block.signed_header.header);

        let mut forks = Vec::with_capacity(witnesses.len());

        for witness in witnesses {
            let mut state = State::new(MemoryStore::new());

            let (witness_block, _) = witness
                .light_client
                .get_or_fetch_block(verified_block.height(), &mut state)?;

            let witness_hash = self.hasher.hash_header(&witness_block.signed_header.header);

            if primary_hash == witness_hash {
                // Hashes match, continue with next witness, if any.
                continue;
            }
        
            // //cross-checking 
            let verified_height = verified_block.height();

            let mut primary_trace=primary.state.get_trace(verified_height);
            primary_trace.reverse();            

        
            for i in primary_trace {          
              
                let (new_witness_block, _) = witness
                .light_client
                .get_or_fetch_block(i.height(), &mut state)?;

                let new_witness_hash = self.hasher.hash_header(&new_witness_block.signed_header.header);
                        
                let new_primary_hash = self.hasher.hash_header(&i.signed_header.header);


                if new_primary_hash != new_witness_hash { 
                    forks.push(Fork::Forked {
                        primary: verified_block.clone(),
                        witness: witness_block.clone(),
                   });      
                    // Hashes match, continue with next height, if any.
                    //trusted_height = *i; 
                   // continue;
                }
    
                    
            }
        }
        if forks.is_empty() {
            Ok(ForkDetection::NotDetected)
        } else {
            Ok(ForkDetection::Detected(forks))
        }
    }
}
