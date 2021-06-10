use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use subtle_encoding::hex;

use tendermint::public_key::PublicKey;
use tendermint_p2p::supervisor::Supervisor;
use tendermint_p2p::transport::BindInfo;

use crate::p2p::transport;

const EXAMPLE_SECRET_KEY: &str = "F7FEB0B5BA0760B2C58893E329475D1EA81781DD636E37144B6D599AD38AA825";

#[test]
fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let transport = transport::Memory {};
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0));
    let info = BindInfo {
        addr,
        advertise_addrs: vec![addr],
        public_key: PublicKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_SECRET_KEY)?).unwrap(),
    };

    Supervisor::run(transport, info)?;

    Ok(())
}
