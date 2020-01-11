//! Main entry point for LightNode

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use light_node::application::APPLICATION;

/// Boot LightNode
fn main() {
    abscissa_core::boot(&APPLICATION);
}
