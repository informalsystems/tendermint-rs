use std::convert::TryFrom;

use chacha20poly1305::{aead::NewAead, ChaCha20Poly1305};
use ed25519_consensus::SigningKey;
use merlin::Transcript;
use rand_core::OsRng;
use subtle::ConstantTimeEq;
use x25519_dalek::{EphemeralSecret, PublicKey as EphemeralPublic};

use tendermint_proto as proto;

use crate::error::Error;
use crate::secret_connection::public_key::PublicKey;
use crate::secret_connection::sort32;
use crate::secret_connection::Kdf;
use crate::secret_connection::Version;

/// Handshake is a process of establishing the `SecretConnection` between two peers.
/// [Specification](https://github.com/tendermint/spec/blob/master/spec/p2p/peer.md#authenticated-encryption-handshake)
pub struct Handshake<S> {
    protocol_version: Version,
    pub(crate) state: S,
}

/// Handshake states

/// `AwaitingEphKey` means we're waiting for the remote ephemeral pubkey.
pub struct AwaitingEphKey {
    local_privkey: SigningKey,
    local_eph_privkey: Option<EphemeralSecret>,
}

/// `AwaitingAuthSig` means we're waiting for the remote authenticated signature.
pub struct AwaitingAuthSig {
    sc_mac: [u8; 32],
    kdf: Kdf,
    pub(crate) recv_cipher: ChaCha20Poly1305,
    pub(crate) send_cipher: ChaCha20Poly1305,
    pub(crate) local_signature: ed25519_consensus::Signature,
}

#[allow(clippy::use_self)]
impl Handshake<AwaitingEphKey> {
    /// Initiate a handshake.
    #[must_use]
    pub fn new(local_privkey: SigningKey, protocol_version: Version) -> (Self, EphemeralPublic) {
        // Generate an ephemeral key for perfect forward secrecy.
        let local_eph_privkey = EphemeralSecret::new(&mut OsRng);
        let local_eph_pubkey = EphemeralPublic::from(&local_eph_privkey);

        (
            Self {
                protocol_version,
                state: AwaitingEphKey {
                    local_privkey,
                    local_eph_privkey: Some(local_eph_privkey),
                },
            },
            local_eph_pubkey,
        )
    }

    /// Performs a Diffie-Hellman key agreement and creates a local signature.
    /// Transitions Handshake into `AwaitingAuthSig` state.
    ///
    /// # Errors
    ///
    /// * if protocol order was violated, e.g. handshake missing
    /// * if challenge signing fails
    pub fn got_key(
        &mut self,
        remote_eph_pubkey: EphemeralPublic,
    ) -> Result<Handshake<AwaitingAuthSig>, Error> {
        let local_eph_privkey = match self.state.local_eph_privkey.take() {
            Some(key) => key,
            None => return Err(Error::missing_secret()),
        };
        let local_eph_pubkey = EphemeralPublic::from(&local_eph_privkey);

        // Compute common shared secret.
        let shared_secret = EphemeralSecret::diffie_hellman(local_eph_privkey, &remote_eph_pubkey);

        let mut transcript = Transcript::new(b"TENDERMINT_SECRET_CONNECTION_TRANSCRIPT_HASH");

        // Reject all-zero outputs from X25519 (i.e. from low-order points)
        //
        // See the following for information on potential attacks this check
        // aids in mitigating:
        //
        // - https://github.com/tendermint/kms/issues/142
        // - https://eprint.iacr.org/2019/526.pdf
        if shared_secret.as_bytes().ct_eq(&[0x00; 32]).unwrap_u8() == 1 {
            return Err(Error::low_order_key());
        }

        // Sort by lexical order.
        let local_eph_pubkey_bytes = *local_eph_pubkey.as_bytes();
        let (low_eph_pubkey_bytes, high_eph_pubkey_bytes) =
            sort32(local_eph_pubkey_bytes, *remote_eph_pubkey.as_bytes());

        transcript.append_message(b"EPHEMERAL_LOWER_PUBLIC_KEY", &low_eph_pubkey_bytes);
        transcript.append_message(b"EPHEMERAL_UPPER_PUBLIC_KEY", &high_eph_pubkey_bytes);
        transcript.append_message(b"DH_SECRET", shared_secret.as_bytes());

        // Check if the local ephemeral public key was the least, lexicographically sorted.
        let loc_is_least = local_eph_pubkey_bytes == low_eph_pubkey_bytes;

        let kdf = Kdf::derive_secrets_and_challenge(shared_secret.as_bytes(), loc_is_least);

        let mut sc_mac: [u8; 32] = [0; 32];

        transcript.challenge_bytes(b"SECRET_CONNECTION_MAC", &mut sc_mac);

        // Sign the challenge bytes for authentication.
        let local_signature = if self.protocol_version.has_transcript() {
            self.state.local_privkey.sign(&sc_mac)
        } else {
            self.state.local_privkey.sign(&kdf.challenge)
        };

        Ok(Handshake {
            protocol_version: self.protocol_version,
            state: AwaitingAuthSig {
                sc_mac,
                recv_cipher: ChaCha20Poly1305::new(&kdf.recv_secret.into()),
                send_cipher: ChaCha20Poly1305::new(&kdf.send_secret.into()),
                kdf,
                local_signature,
            },
        })
    }
}

impl Handshake<AwaitingAuthSig> {
    /// Returns a verified pubkey of the remote peer.
    ///
    /// # Errors
    ///
    /// * if signature scheme isn't supported
    pub fn got_signature(
        &mut self,
        auth_sig_msg: proto::p2p::AuthSigMessage,
    ) -> Result<PublicKey, Error> {
        let pk_sum = auth_sig_msg
            .pub_key
            .and_then(|key| key.sum)
            .ok_or_else(Error::missing_key)?;

        let remote_pubkey = match pk_sum {
            proto::crypto::public_key::Sum::Ed25519(ref bytes) => {
                ed25519_consensus::VerificationKey::try_from(&bytes[..])
                    .map_err(|_| Error::signature())
            }
            _ => Err(Error::unsupported_key()),
        }?;

        let remote_sig = ed25519_consensus::Signature::try_from(auth_sig_msg.sig.as_slice())
            .map_err(|_| Error::signature())?;

        if self.protocol_version.has_transcript() {
            remote_pubkey
                .verify(&remote_sig, &self.state.sc_mac)
                .map_err(|_| Error::signature())?;
        } else {
            remote_pubkey
                .verify(&remote_sig, &self.state.kdf.challenge)
                .map_err(|_| Error::signature())?;
        }

        // We've authorized.
        Ok(remote_pubkey.into())
    }
}
