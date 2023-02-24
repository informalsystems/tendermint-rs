use crate::Error;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VerificationKey([u8; 32]);

impl core::fmt::Display for VerificationKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl core::fmt::Debug for VerificationKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Display>::fmt(self, f)
    }
}

impl VerificationKey {
    #[allow(dead_code)]
    pub(super) fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&'_ [u8]> for VerificationKey {
    type Error = Error;

    fn try_from(slice: &'_ [u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Error::invalid_key("invalid ed25519 key length".into()));
        }
        let mut bytes = [0u8; 32];
        bytes[..].copy_from_slice(slice);
        Ok(Self(bytes))
    }
}

#[cfg(feature = "rust-crypto")]
impl TryFrom<VerificationKey> for ed25519_consensus::VerificationKey {
    type Error = Error;

    fn try_from(src: VerificationKey) -> Result<Self, Error> {
        ed25519_consensus::VerificationKey::try_from(src.0)
            .map_err(|_| Error::invalid_key("malformed Ed25519 public key".into()))
    }
}
