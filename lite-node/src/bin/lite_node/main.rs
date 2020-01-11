//! Main entry point for LiteNode

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use lite_node::application::APPLICATION;

/// Boot LiteNode
fn main() {
    abscissa_core::boot(&APPLICATION);
}
