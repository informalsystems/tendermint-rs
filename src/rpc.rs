use prost::Message;
use std::io::{self, Read};
use std::io::{Error, ErrorKind};
use types::{PoisonPillMsg, PubKeyMsg, SignHeartbeatMsg, SignProposalMsg, SignVoteMsg};

/// Requests to the KMS
pub enum Request {
    /// Sign the given message
    SignHeartbeat(SignHeartbeatMsg),
    SignProposal(SignProposalMsg),
    SignVote(SignVoteMsg),
    ShowPublicKey(PubKeyMsg),

    /// Instruct the KMS to terminate
    PoisonPill(PoisonPillMsg),
}

impl Request {
    /// Read a request from the given readable
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = vec![];
        println!("reading ....");
        r.read_to_end(&mut buf)?;
        println!("got request!!");
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
        if let Ok(pill) = PoisonPillMsg::decode(&buf) {
            return Ok(Request::PoisonPill(pill));
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            "Received unknown RPC message.",
        ))
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
