//! JSON-RPC error types

use core::time::Duration;

use flex_error::{define_error, DefaultTracer, DisplayError, DisplayOnly, ErrorMessageTracer};

use crate::{prelude::*, response_error::ResponseError, rpc_url::Url};

#[cfg(feature = "reqwest")]
type ReqwestError = flex_error::DisplayOnly<reqwest::Error>;

#[cfg(not(feature = "reqwest"))]
type ReqwestError = flex_error::NoSource;

#[cfg(feature = "tokio")]
type JoinError = flex_error::DisplayOnly<tokio::task::JoinError>;

#[cfg(not(feature = "tokio"))]
type JoinError = flex_error::NoSource;

#[cfg(feature = "async-tungstenite")]
type TungsteniteError = flex_error::DisplayOnly<async_tungstenite::tungstenite::Error>;

#[cfg(not(feature = "async-tungstenite"))]
type TungsteniteError = flex_error::NoSource;

define_error! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    Error {
        Response
            [ DisplayError<ResponseError> ]
            | _ | { "response error" },

        Io
            [ DisplayOnly<std::io::Error> ]
            | _ | { "I/O error" },

        Http
            [ ReqwestError ]
            | _ | { "HTTP error" },

        InvalidProxy
            [ ReqwestError ]
            | _ | { "Invalid proxy configuration" },

        InvalidParams
            {
                message: String
            }
            | e | {
                format_args!("invalid params error: {}", e.message)
            },

        WebSocket
            {
                message: String
            }
            [ TungsteniteError ]
            | e | {
                format_args!("web socket error: {}", e.message)
            },

        WebSocketTimeout
            {
                timeout: Duration
            }
            | e | {
                format_args!("reading from WebSocket connection timed out after {} seconds",
                    e.timeout.as_secs())
            },

        MethodNotFound
            {
                method: String
            }
            | e | {
                format_args!("method not found: {}", e.method)
            },

        Parse
            {
                reason: String
            }
            | e | {
                format_args!("parse error: {}", e.reason)
            },

        Server
            {
                reason: String
            }
            | e | {
                format_args!("server error: {}", e.reason)
            },

        ClientInternal
            {
                reason: String
            }
            | e | {
                format_args!("client internal error: {}", e.reason)
            },

        Timeout
            {
                duration: Duration
            }
            | e | {
                format_args!(
                    "timed out waiting for healthy response after {}ms",
                    e.duration.as_millis()
                )
            },

        ChannelSend
            | _ | { "failed to send message to internal channel" },

        InvalidUrl
            { url: Url }
            | e | {
                format_args!(
                    "cannot use URL {} with HTTP clients",
                    e.url
                )
            },

        Tendermint
            [ tendermint::Error ]
            | _ | { "tendermint error" },

        ParseInt
            [ DisplayOnly<core::num::ParseIntError> ]
            | _ | { "error parsing integer" },

        OutOfRange
            [ DisplayOnly<core::num::TryFromIntError> ]
            | _ | { "number out of range" },

        InvalidNetworkAddress
            | _ | { "only TCP-based node addresses are supported" },

        MismatchResponse
            | _ | { "no matching response for incoming request" },

        UnrecognizedEventType
            {
                event_type: String
            }
            | e | {
                format_args!("unrecognized event type: {}", e.event_type)
            },

        Serde
            [ DisplayOnly<serde_json::Error> ]
            | _ | { "serde parse error" },

        ParseUrl
            [ DisplayOnly<url::ParseError> ]
            | _ | { "parse error" },

        Tungstenite
            [ TungsteniteError ]
            | _ | { "tungstenite error" },

        Join
            [ JoinError ]
            | _ | { "join error" },

        MalformedJson
            | _ | { "server returned malformatted JSON (no 'result' or 'error')" },

        UnsupportedScheme
            {
                scheme: String
            }
            | e | {
                format_args!("unsupported scheme: {}", e.scheme)
            },

        UnsupportedRpcVersion
            {
                version: String,
                supported: String
            }
            | e | {
                format_args!("server RPC version unsupported: '{}' (only '{}' supported)",
                    e.version, e.supported)
            },

        InvalidTendermintVersion
            {
                version: String,
            }
            | e | {
                format_args!("invalid Tendermint version reported by the node: {}", e.version)
            },

        UnsupportedTendermintVersion
            {
                version: String,
            }
            | e | {
                format_args!("unsupported Tendermint version reported by the node: {}", e.version)
            },
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error(
            self.detail().clone(),
            DefaultTracer::new_message(self.trace()),
        )
    }
}

#[cfg(feature = "tokio")]
impl Error {
    pub fn send<T>(_: tokio::sync::mpsc::error::SendError<T>) -> Error {
        Error::channel_send()
    }
}
