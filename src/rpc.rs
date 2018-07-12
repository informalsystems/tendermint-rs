// TODO: replace this with amino

use std::io::{self, Error, ErrorKind, Read};
use types::{SignHeartbeatMsg, SignProposalMsg, SignVoteMsg};
/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignHeartbeat(SignHeartbeatMsg),
    SignProposal(SignProposalMsg),
    SignVote(SignVoteMsg),
    ShowPublicKey(),

    /// Instruct the KMS to terminate
    #[cfg(debug_assertions)]
    PoisonPill,
}

impl Request {
    /// Read a request from the given readable
    #[allow(dead_code)]
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = vec![];
        r.read(&mut buf)?;
        if let Ok(hb) = SignHeartbeatMsg::decode(&buf){
            return Ok(Request::SignHeartbeat(hb));
        }
        if let Ok(vote) = SignVoteMsg::decode(&buf) {
            return Ok(Request::SignVote(vote));
        }
        if let Ok(prop) = SignProposalMsg::decode(&buf) {
            return Ok(Request::SignProposal(prop));
        }

        // TODO: don't unwrap, but really... switch to amino
        Err(Error::new(ErrorKind::Other, "Invalid RPC message"))
    }

    /// Serialize a request as a byte vector
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<u8> {
        // TODO: don't unwrap, but really... switch to amino

        let mut body = match self {
            Request::SignProposal(prop) => prop.encode(),
        };

        let mut msg = vec![body.len() as u8];
        msg.append(body.as_mut());
        msg
    }
}

/// Responses from the KMS
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Signature response
    SignedHeartBeat(SignHeartbeatMsg),
    SignedVote(SignVoteMsg),
    SignedProposal(SignProposalMsg),
    PublicKey(),
}
