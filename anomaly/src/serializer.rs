//! `serde` serialization support - allows for producing e.g. JSON
//! serializations of errors.

use crate::{BoxError, Context, Error};
use backtrace::Backtrace;
use serde::{ser, Serialize};
use std::{
    error::Error as _,
    fmt::{Debug, Display},
    ops::Deref,
};

/// Serializable error type, useful for sending to exception reporting systems.
#[derive(Clone, Debug, Serialize)]
pub struct SerializedError<Kind>
where
    Kind: Clone + Debug + Serialize,
{
    /// Name of the error kind type
    #[serde(rename = "type")]
    pub type_name: String,

    /// Kind of error
    pub kind: Kind,

    /// Error message
    pub msg: String,

    /// Error source
    pub source: Option<String>,

    /// Error backtrace
    pub backtrace: Option<Backtrace>,
}

impl<Kind> From<&Context<Kind>> for SerializedError<Kind>
where
    Kind: Clone + Debug + Display + Into<BoxError> + Serialize,
{
    fn from(context: &Context<Kind>) -> Self {
        Self {
            type_name: std::any::type_name::<Kind>().to_owned(),
            kind: context.kind().clone(),
            msg: context.kind().to_string(),
            source: context.source().map(ToString::to_string),
            backtrace: context.backtrace().cloned(),
        }
    }
}

impl<Kind> Serialize for Context<Kind>
where
    Kind: Clone + Debug + Display + Into<BoxError> + Serialize,
{
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SerializedError::from(self).serialize(serializer)
    }
}

impl<Kind> Serialize for Error<Kind>
where
    Kind: Clone + Debug + Display + Into<BoxError> + Serialize,
{
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SerializedError::from(self.deref()).serialize(serializer)
    }
}
