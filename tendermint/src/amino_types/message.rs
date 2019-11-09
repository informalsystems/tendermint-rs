/// A wrapper whose sole purpose is to extend the original
/// prost::Message trait with a few helper functions in order to reduce boiler-plate code
/// (and without modifying the prost-amino dependency).
pub struct AminoMessage<M>(M); // Note: we have to introduce a new type in order to add methods like bytes_vec or impl Into<Vec<u8>>

impl<M> AminoMessage<M>
where
    M: prost_amino::Message,
{
    /// Returns a wrapper for the given prost-amino message
    pub fn new(m: M) -> AminoMessage<M> {
        AminoMessage(m)
    }
    /// Directly amino encode a prost-amino message into a freshly created Vec<u8>.
    /// This can be useful when passing those bytes directly to a hasher, or,
    /// to reduce boiler plate code when working with the encoded bytes.
    ///
    /// Warning: Only use this method, if you are in control what will be encoded.
    /// If there is an encoding error, this method will panic.
    pub fn bytes_vec(m: M) -> Vec<u8> {
        AminoMessage(m).into()
    }
}

impl<M> Into<Vec<u8>> for AminoMessage<M>
where
    M: prost_amino::Message,
{
    /// Convert a wrapped prost-amino message directly into a byte vector which contains the amino
    /// encoding.
    ///
    /// Warning: Only use this method, if you are in control what will be encoded. If there is an
    /// encoding error, this method will panic.
    fn into(self) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.0.encoded_len());
        self.0.encode(&mut res).unwrap();
        res
    }
}
