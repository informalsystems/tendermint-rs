#[derive(Clone, PartialEq, Message)]
pub struct RemoteError {
    #[prost(sint32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub description: String,
}
