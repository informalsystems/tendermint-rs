use std::io::{self, Read};
use types::{PubKeyMsg, SignHeartbeatMsg, SignProposalMsg, SignVoteMsg};

use prost::Message;

/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignHeartbeat(SignHeartbeatMsg),
    SignProposal(SignProposalMsg),
    SignVote(SignVoteMsg),
    ShowPublicKey(PubKeyMsg),

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
        if let Ok(prop) = PubKeyMsg::decode(&buf) {
            return Ok(Request::ShowPublicKey(prop));
        }

        // TODO(ismail) PoisonPill is missing here
        println!("TODO: we just assume we want to terminate here ...");
        Ok(Request::PoisonPill)

        // TODO: don't unwrap, but really... switch to amino
        // Err(Error::new(ErrorKind::Other, "Invalid RPC message"))
    }
}

/// Responses from the KMS
pub enum Response {
    /// Signature response
    SignedHeartBeat(SignHeartbeatMsg),
    SignedVote(SignVoteMsg),
    SignedProposal(SignProposalMsg),
    PublicKey(PubKeyMsg),
}
