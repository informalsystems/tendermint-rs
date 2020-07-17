//! Main entry point for light-node

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use tendermint_light_node::application::APPLICATION;

/// Boot LightNode
fn main() {
    abscissa_core::boot(&APPLICATION);
}
