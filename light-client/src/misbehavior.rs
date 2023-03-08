mod detect;
mod error;
mod evidence;

pub use detect::handle_conflicting_headers;
pub use error::DetectorError;
pub use tendermint::evidence::LightClientAttackEvidence;
