//! `/genesis` endpoint JSON-RPC wrapper

use core::{fmt, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tendermint::Genesis;

/// Get the genesis state for the current chain
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request<AppState>(#[serde(skip)] PhantomData<AppState>);

impl<AppState> Default for Request<AppState> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<AppState> crate::Request for Request<AppState>
where
    AppState: fmt::Debug + Serialize + DeserializeOwned + Send,
{
    type Response = Response<AppState>;

    fn method(&self) -> crate::Method {
        crate::Method::Genesis
    }
}

impl<AppState> crate::SimpleRequest for Request<AppState> where
    AppState: fmt::Debug + Serialize + DeserializeOwned + Send
{
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response<AppState> {
    /// Genesis data
    pub genesis: Genesis<AppState>,
}

impl<AppState> crate::Response for Response<AppState> where AppState: Serialize + DeserializeOwned {}
