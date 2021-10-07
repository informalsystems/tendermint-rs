use flex_error::{define_error, DisplayOnly};
use std::io::Error as IoError;
use tendermint::Error as TendermintError;

define_error! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    Error {
        Io
            [ DisplayOnly<IoError> ]
            |_| { format_args!("I/O error") },

        FileIo
            { path: String }
            [ DisplayOnly<IoError> ]
            |e| { format_args!("failed to open file: {}", e.path) },

        Parse
            { data: String }
            | e | { format_args!("error parsing data: {}", e.data) },

        SerdeJson
            [ DisplayOnly<serde_json::Error> ]
            |_| { format_args!("serde json error") },

        Toml
            [ DisplayOnly<toml::de::Error> ]
            |_| { format_args!("toml de error") },

        ParseUrl
            [ DisplayOnly<url::ParseError> ]
            |_| { format_args!("error parsing url error") },

        Tendermint
            [ TendermintError ]
            |_| { format_args!("tendermint error") },
    }
}
