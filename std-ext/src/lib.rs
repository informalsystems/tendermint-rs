//! Extensions to the [Rust standard library][std] for use by [tendermint-rs].
//!
//! [std]: https://doc.rust-lang.org/std/
//! [tendermint-rs]: https://github.com/informalsystems/tendermint-rs/

mod try_clone;

pub use try_clone::TryClone;
