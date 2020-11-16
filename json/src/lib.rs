//! tendermint-json contains representations of all [Tendermint] data structures
//! such that they can be serialized/deserialized to/from JSON (and other
//! formats supported by [serde]).
//!
//! [Tendermint]: https://tendermint.com
//! [serde]: https://serde.rs/

pub mod abci;
pub mod bits;
pub mod block;
pub mod crypto;
pub mod evidence;
pub mod serializers;
pub mod time;
pub mod version;
pub mod vote;
