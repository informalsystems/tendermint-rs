//! ABCI application interface.

#[cfg(feature = "echo-app")]
pub mod echo;

use tendermint_proto::abci::request::Value;
use tendermint_proto::abci::{response, Request, RequestEcho, Response, ResponseEcho};

/// An ABCI application.
pub trait Application: Send + Clone + 'static {
    fn echo(&self, request: RequestEcho) -> ResponseEcho {
        ResponseEcho {
            message: request.message,
        }
    }

    /// Executes the relevant application method based on the type of the
    /// request, and produces the corresponding response.
    fn handle(&self, request: Request) -> Response {
        Response {
            value: Some(match request.value.unwrap() {
                Value::Echo(req) => response::Value::Echo(self.echo(req)),
                _ => unimplemented!(),
                // Value::Flush(_) => {}
                // Value::Info(_) => {}
                // Value::SetOption(_) => {}
                // Value::InitChain(_) => {}
                // Value::Query(_) => {}
                // Value::BeginBlock(_) => {}
                // Value::CheckTx(_) => {}
                // Value::DeliverTx(_) => {}
                // Value::EndBlock(_) => {}
                // Value::Commit(_) => {}
                // Value::ListSnapshots(_) => {}
                // Value::OfferSnapshot(_) => {}
                // Value::LoadSnapshotChunk(_) => {}
                // Value::ApplySnapshotChunk(_) => {}
            }),
        }
    }
}
