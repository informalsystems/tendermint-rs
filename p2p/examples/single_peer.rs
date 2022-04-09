use std::io::Read as _;
use std::net::{SocketAddr, TcpStream};

use prost::Message;
use rand_core::OsRng;

use proto::p2p::NodeInfo;
use proto::Protobuf as _;
use tendermint::node;
use tendermint_p2p::secret_connection::{SecretConnection, Version};
use tendermint_proto as proto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = OsRng {};
    let sk = ed25519_consensus::SigningKey::new(&mut rng);
    let io = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], 26656)))?;

    let mut conn = SecretConnection::new(io, sk, Version::V0_35)?;

    dbg!(conn.remote_pubkey());

    let mut buf: [u8; 256] = [0; 256];
    conn.read(&mut buf)?;

    let (len, buf) = unsigned_varint::decode::u64(&buf).unwrap();
    dbg!(len);
    dbg!(buf.len());

    let data = &buf[..len as usize];
    dbg!(&data);

    let node_info = node::Info::decode(data).unwrap();

    dbg!(node_info);

    Ok(())
}
