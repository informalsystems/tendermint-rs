//! Validator errors (returned in a status via gRPC)

use flex_error::{define_error, DetailOnly};

define_error! {
    Error {
        IoError{
            path: String,
        } [DetailOnly<std::io::Error>] |e| {
            format_args!("Error persisting {}", e.path)
        },
        JsonError{
            path_or_msg: String,
        } [DetailOnly<serde_json::Error>] |e| {
            format_args!("Error parsing or serializing validator state {}", e.path_or_msg)
        },
    }
}
