//! Main entry point for the `tmkms` executable

extern crate tmkms;
use tmkms::KmsApplication;

/// Boot the `tmkms` application
fn main() {
    KmsApplication::boot();
}
