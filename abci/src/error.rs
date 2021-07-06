//! tendermint-abci errors

use flex_error::{define_error, DisplayError};
use tendermint_proto::abci::response::Value;

define_error! {
    Error {
        Io
            [ DisplayError<std::io::Error> ]
            | _ | { "I/O error" },

        Encode
            [ DisplayError<prost::EncodeError> ]
            | _ | { "error encoding protocol buffer" },

        Decode
            [ DisplayError<prost::DecodeError> ]
            | _ | { "error encoding protocol buffer" },

        ServerConnectionTerminated
            | _ | { "server connection terminated" },

        MalformedServerResponse
            | _ | { "malformed server response" },

        UnexpectedServerResponseType
            {
                expected: String,
                got: Value,
            }
            | e | {
                format_args!("unexpected server response type: expected {0}, but got {1:?}",
                    e.expected, e.got)
            },

        ChannelSend
            | _ | { "channel send error" },

        ChannelRecv
            [ DisplayError<std::sync::mpsc::RecvError> ]
            | _ | { "channel recv error" },
    }
}

pub fn send_error<T>(_e: std::sync::mpsc::SendError<T>) -> Error {
    channel_send_error()
}
