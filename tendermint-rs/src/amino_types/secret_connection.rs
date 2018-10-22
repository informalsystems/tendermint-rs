//! Message types used by `SecretConnection`

#[derive(Clone, PartialEq, Message)]
pub struct AuthSigMessage {
    #[prost(bytes, tag = "1")]
    pub key: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub sig: Vec<u8>,
}
