//! JSON-RPC error types

use flex_error::{define_error, DisplayError, DisplayOnly};
use http::uri::InvalidUri;
use http::Error as HttpError;
use hyper::Error as HyperError;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;

use crate::response_error::ResponseError;
use crate::rpc_url::Url;

define_error! {
    #[derive(Debug, Clone)]
    Error {
        Response
            [ DisplayError<ResponseError> ]
            | _ | { "response error" },

        Io
            [ DisplayOnly<std::io::Error> ]
            | _ | { "I/O error" },

        Http
            [ DisplayOnly<HttpError> ]
            | _ | { "HTTP error" },

        Hyper
            [ DisplayOnly<HyperError> ]
            | _ | { "HTTP error" },

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
            [ DisplayOnly<tungstenite::Error> ]
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

        InvalidUri
            [ DisplayOnly<InvalidUri> ]
            | _ | { "invalid URI" },

        Tendermint
            [ tendermint::Error ]
            | _ | { "tendermint error" },

        ParseInt
            [ DisplayOnly<std::num::ParseIntError> ]
            | _ | { "error parsing integer" },

        OutOfRange
            [ DisplayOnly<std::num::TryFromIntError> ]
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
            | _ | { "parse error" },

        ParseUrl
            [ DisplayOnly<url::ParseError> ]
            | _ | { "parse error" },

        Tungstenite
            [ DisplayOnly<tungstenite::Error> ]
            | _ | { "tungstenite error" },

        Join
            [ DisplayOnly<tokio::task::JoinError> ]
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

    }
}

pub fn send_error<T>(_: SendError<T>) -> Error {
    channel_send_error()
}
