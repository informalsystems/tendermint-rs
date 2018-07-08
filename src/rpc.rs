// TODO: replace this with amino

use std::io::{self, Read};
use types::{Heartbeat,Proposal,Vote};
use amino::Amino;
/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignHeartbeat(Heartbeat),
    SignProposal(Proposal),
    SignVote(Vote),
    ShowPublicKey(),    

    /// Instruct the KMS to terminate
    #[cfg(debug_assertions)]
    PoisonPill,
}

impl Request {
    /// Read a request from the given readable
    #[allow(dead_code)]
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        // Read the length
        let mut len = [0u8];
        r.read_exact(&mut len)?;

        let mut buf = vec![];
        for _ in 0..len[0] {
            buf.push(0u8);
        }

        r.read_exact(&mut buf)?;

        // TODO: don't unwrap, but really... switch to amino
        Ok()
    }

    /// Serialize a request as a byte vector
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<u8> {
        // TODO: don't unwrap, but really... switch to amino
        let mut body = bincode::serialize(self).unwrap();
        let mut msg = vec![body.len() as u8];
        msg.append(body.as_mut());
        msg
    }
}

/// Sign the given opaque message with the given public key
#[derive(Serialize, Deserialize, Debug)]
pub struct SignRequest {
    /// Public key to use to sign the request
    pub public_key: Vec<u8>,

    /// Message to be signed
    pub msg: Vec<u8>,
}

/// Responses from the KMS
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Signature response
    Sign(SignResponse),
}

impl Response {
    /// Read a response from the given readable
    #[allow(dead_code)]
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        // Read the length
        let mut len = [0u8];
        r.read_exact(&mut len)?;

        let mut buf = vec![];
        for _ in 0..len[0] {
            buf.push(0u8);
        }

        r.read_exact(&mut buf)?;

        // TODO: don't unwrap, but really... switch to amino
        Ok(bincode::deserialize(&buf).unwrap())
    }

    /// Serialize this response as a byte vector
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<u8> {
        let mut body = bincode::serialize(self).unwrap();
        let mut msg = vec![body.len() as u8];
        msg.append(body.as_mut());
        msg
    }
}

/// Response containing a signed message
#[derive(Serialize, Deserialize, Debug)]
pub struct SignResponse {
    /// Resulting signature
    pub sig: Vec<u8>,
}
