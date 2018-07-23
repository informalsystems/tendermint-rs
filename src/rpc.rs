// TODO: replace this with amino

use prost::Message;
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
        if let Ok(hb) = SignHeartbeatMsg::decode(&buf) {
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
}

/// Responses from the KMS
pub enum Response {
    /// Signature response
    SignedHeartBeat(SignHeartbeatMsg),
    SignedVote(SignVoteMsg),
    SignedProposal(SignProposalMsg),
    PublicKey(),
}
