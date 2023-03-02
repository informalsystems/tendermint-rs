//! `/genesis` endpoint JSON-RPC wrapper

use core::{fmt, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tendermint::Genesis;

use crate::{dialect::Dialect, request::RequestMessage};

/// Get the genesis state for the current chain
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request<AppState>(#[serde(skip)] PhantomData<AppState>);

impl<AppState> Default for Request<AppState> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<AppState> RequestMessage for Request<AppState>
where
    AppState: Serialize + DeserializeOwned,
{
    fn method(&self) -> crate::Method {
        crate::Method::Genesis
    }
}

impl<AppState, S> crate::Request<S> for Request<AppState>
where
    AppState: fmt::Debug + Serialize + DeserializeOwned + Send,
    S: Dialect,
{
    type Response = Response<AppState>;
}

impl<AppState, S> crate::SimpleRequest<S> for Request<AppState>
where
    AppState: fmt::Debug + Serialize + DeserializeOwned + Send,
    S: Dialect,
{
    type Output = Response<AppState>;
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response<AppState> {
    /// Genesis data
    pub genesis: Genesis<AppState>,
}

impl<AppState> crate::Response for Response<AppState> where AppState: Serialize + DeserializeOwned {}
