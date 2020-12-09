//! ABCI echo request

use tendermint_proto::abci::{RequestEcho, ResponseEcho};

pub struct Request(RequestEcho);

impl super::Request for Request {
    type Message = RequestEcho;
    type Response = Response;
}

pub struct Response(ResponseEcho);

impl super::Response for Response {
    type Message = ResponseEcho;
}
