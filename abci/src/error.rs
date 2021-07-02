//! tendermint-abci errors

use flex_error::define_error;
use tendermint_proto::abci::response::Value;

define_error! {
    Error {
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
            | _ | { "channel recv error" },
    }
}
