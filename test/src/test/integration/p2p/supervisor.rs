use tendermint_p2p::supervisor::Supervisor;

use crate::p2p::transport;

#[test]
fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let transport = transport::Memory {};
    let info = todo!();

    Supervisor::run(transport, info)?;

    Ok(())
}
