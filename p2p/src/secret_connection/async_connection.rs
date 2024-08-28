use super::{
    decrypt, encrypt, proto, EphemeralPublic, Error, Handshake, Nonce, PublicKey, ReceiveState,
    SendState, Version, DATA_LEN_SIZE, DATA_MAX_SIZE, TAG_SIZE, TOTAL_FRAME_SIZE,
};
use std::slice;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

/// Encrypted connection between peers in a Tendermint network, implemented asynchronously using
/// Tokio as the underlying async runtime.
pub struct AsyncSecretConnection {
    tcp_stream: TcpStream,
    protocol_version: Version,
    remote_pubkey: Option<PublicKey>,
    send_state: SendState,
    recv_state: ReceiveState,
}

impl AsyncSecretConnection {
    /// Open a TCP connection to the given socket address, performing a `SecretConnection` handshake
    /// and returning a new client upon success.
    ///
    /// # Errors
    ///
    /// * if TCP connection fails
    /// * if sharing of the pubkey fails
    /// * if sharing of the signature fails
    /// * if receiving the signature fails
    pub async fn connect_tcp<A: ToSocketAddrs>(
        addr: A,
        local_privkey: ed25519_consensus::SigningKey,
        protocol_version: Version,
    ) -> Result<Self, Error> {
        let mut tcp_stream = TcpStream::connect(addr).await?;

        // Start a handshake process.
        let local_pubkey = PublicKey::from(&local_privkey);
        let (mut h, local_eph_pubkey) = Handshake::new(local_privkey, protocol_version);

        // Write local ephemeral pubkey and receive one too.
        let remote_eph_pubkey =
            exchange_eph_pubkey(&mut tcp_stream, &local_eph_pubkey, protocol_version).await?;

        // Compute a local signature (also recv_cipher & send_cipher)
        let h = h.got_key(remote_eph_pubkey)?;

        let mut sc = Self {
            tcp_stream,
            protocol_version,
            remote_pubkey: None,
            send_state: SendState {
                cipher: h.state.send_cipher.clone(),
                nonce: Nonce::default(),
            },
            recv_state: ReceiveState {
                cipher: h.state.recv_cipher.clone(),
                nonce: Nonce::default(),
                buffer: vec![],
            },
        };

        // Share each other's pubkey & challenge signature.
        // NOTE: the data must be encrypted/decrypted using ciphers.
        let auth_sig_msg = match local_pubkey {
            PublicKey::Ed25519(ref pk) => {
                sc.share_auth_signature(pk, &h.state.local_signature)
                    .await?
            },
        };

        // Authenticate remote pubkey.
        let remote_pubkey = h.got_signature(auth_sig_msg)?;

        // All good!
        sc.remote_pubkey = Some(remote_pubkey);
        Ok(sc)
    }

    /// Returns the remote pubkey. Panics if there's no key.
    ///
    /// # Panics
    /// * if the remote pubkey is not initialized.
    pub fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey.expect("remote_pubkey uninitialized")
    }

    async fn share_auth_signature(
        &mut self,
        pubkey: &ed25519_consensus::VerificationKey,
        local_signature: &ed25519_consensus::Signature,
    ) -> Result<proto::p2p::AuthSigMessage, Error> {
        let buf = self
            .protocol_version
            .encode_auth_signature(pubkey, local_signature);

        self.write_all(&buf).await?;

        let auth_sig = self.read_chunk().await?;
        debug_assert_eq!(
            auth_sig.len(),
            self.protocol_version.auth_sig_msg_response_len()
        );
        self.protocol_version.decode_auth_signature(&buf)
    }

    async fn read_chunk<'a>(&'a mut self) -> Result<Vec<u8>, Error> {
        debug_assert!(self.recv_state.buffer.is_empty());

        let mut sealed_frame = [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
        self.tcp_stream.read_exact(&mut sealed_frame).await?;

        // decrypt the frame
        let mut frame = [0_u8; TOTAL_FRAME_SIZE];
        decrypt(
            &sealed_frame,
            &self.recv_state.cipher,
            &self.recv_state.nonce,
            &mut frame,
        )?;

        self.recv_state.nonce.increment();
        // end decryption

        let chunk_length = u32::from_le_bytes(frame[..4].try_into().expect("chunk framing failed"));

        if chunk_length as usize > DATA_MAX_SIZE {
            return Err(std::io::Error::new(
                 std::io::ErrorKind::Other,
                 format!("chunk is too big: {chunk_length}! max: {DATA_MAX_SIZE}"),
            ).into());
        }

        let mut chunk = vec![0; chunk_length as usize];
        chunk.clone_from_slice(
            &frame[DATA_LEN_SIZE
                ..(DATA_LEN_SIZE
                    .checked_add(chunk_length as usize)
                    .expect("chunk size addition overflow"))],
        );

        Ok(chunk)
    }

    /// Write encrypted data to the underlying TCP socket.
    pub async fn write_all<'a>(&'a mut self, src: &'a [u8]) -> Result<usize, Error> {
        let mut n = 0_usize;

        for chunk in src.chunks(DATA_MAX_SIZE) {
            let mut sealed_frame = [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
            encrypt(
                chunk,
                &self.send_state.cipher,
                &self.send_state.nonce,
                &mut sealed_frame,
            )?;

            self.send_state.nonce.increment();
            // end encryption

            self.tcp_stream.write_all(&sealed_frame).await?;
            n = n
                .checked_add(chunk.len())
                .expect("overflow when adding chunk lengths");
        }

        Ok(n)
    }
}

/// Returns `remote_eph_pubkey`
async fn exchange_eph_pubkey(
    tcp_stream: &mut TcpStream,
    local_eph_pubkey: &EphemeralPublic,
    protocol_version: Version,
) -> Result<EphemeralPublic, Error> {
    // Send our pubkey and receive theirs in tandem.
    // TODO(ismail): on the go side this is done in parallel, here we do send and receive after
    tcp_stream
        .write_all(&protocol_version.encode_initial_handshake(local_eph_pubkey))
        .await?;

    let mut response_len = 0_u8;
    tcp_stream
        .read_exact(slice::from_mut(&mut response_len))
        .await?;

    let mut buf = vec![0; response_len as usize];
    tcp_stream.read_exact(&mut buf).await?;
    protocol_version.decode_initial_handshake(&buf)
}
