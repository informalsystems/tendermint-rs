use std::cmp;
use std::io;
use std::mem;
use std::future::Future;
use std::pin::Pin;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use ed25519_consensus::{Signature, SigningKey, VerificationKey};
use futures::io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _};
use futures::ready;
use x25519_dalek::PublicKey as EphemeralPublicKey;

use tendermint_proto as proto;

use crate::error::Error;
use crate::secret_connection::decrypt;
use crate::secret_connection::public_key::PublicKey;
use crate::secret_connection::DATA_LEN_SIZE;
use crate::secret_connection::TAG_SIZE;
use crate::secret_connection::TOTAL_FRAME_SIZE;
use crate::secret_connection::{Handshake, Nonce, ReceiveState, SendState, Version};

pub struct AsyncSecrectConnection<Io> {
    protocol_version: Version,
    remote_pubkey: PublicKey,

    io: Io,

    send_state: SendState,
    recv_state: ReceiveState,
    terminate: Arc<AtomicBool>,
}

impl<Io> AsyncSecrectConnection<Io>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new(
        mut io: Io,
        local_signing_key: SigningKey,
        protocol_version: Version,
    ) -> Result<Self, Error> {
        let local_pubkey = PublicKey::from(&local_signing_key);
        let (mut handshake, local_eph_pubkey) = Handshake::new(local_signing_key, protocol_version);

        let remote_eph_pubkey =
            share_eph_pubkey(&mut io, &local_eph_pubkey, protocol_version).await?;

        let mut handshake = handshake.got_key(remote_eph_pubkey)?;

        let auth_sig_msg = match local_pubkey {
            PublicKey::Ed25519(ref pk) => {
                share_auth_signature(
                    &mut io,
                    protocol_version,
                    pk,
                    &handshake.state.local_signature,
                )
                .await?
            }
        };

        let remote_pubkey = handshake.got_signature(auth_sig_msg)?;

        Ok(Self {
            protocol_version,
            remote_pubkey,

            io,

            send_state: SendState {
                cipher: handshake.state.send_cipher.clone(),
                nonce: Nonce::default(),
            },
            recv_state: ReceiveState {
                cipher: handshake.state.recv_cipher.clone(),
                nonce: Nonce::default(),
                buffer: vec![],
            },
            terminate: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn remote_pubkey(&self) -> PublicKey {
        self.remote_pubkey
    }
}

impl<Io> AsyncRead for AsyncSecrectConnection<Io>
where
    Io: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.terminate.load(Ordering::SeqCst) {
            return Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Other,
                "secret connection was terminated elsewhere by previous error",
            )));
        }

        if !self.recv_state.buffer.is_empty() {
            let n = cmp::min(buf.len(), self.recv_state.buffer.len());
            buf.copy_from_slice(&self.recv_state.buffer[..n]);
            let mut leftover_portion = vec![
                0;
                self.recv_state
                    .buffer
                    .len()
                    .checked_sub(n)
                    .expect("failed to calculate leftover")
            ];
            leftover_portion.clone_from_slice(&self.recv_state.buffer[n..]);
            self.recv_state.buffer = leftover_portion;

            return Poll::Ready(Ok(n));
        }

        let sealed_frame = &mut [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE];
        Future::poll(self.io.read_exact(sealed_frame).as_mut(), cx)

        while !sealed_frame.is_empty() {
            let n = ready!(Pin::new(&mut self.io).poll_read(cx, sealed_frame))?;
            {
                let (_, rest) =
                    mem::replace(sealed_frame, [0_u8; TAG_SIZE + TOTAL_FRAME_SIZE]).split_at_mut(n);
                sealed_frame = rest;
            }

            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()));
            }
        }

        let mut frame = [0_u8; TOTAL_FRAME_SIZE];
        let res = decrypt(
            sealed_frame,
            &self.recv_state.cipher,
            &self.recv_state.nonce,
            &mut frame,
        );

        if let Err(err) = res {
            return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, err.to_string())));
        }

        self.recv_state.nonce.increment();

        let chunk_length =
            u32::from_le_bytes(frame[..4].try_into().expect("failed to frame chunk"));

        let mut chunk = vec![0; chunk_length as usize];
        chunk.clone_from_slice(
            &frame[DATA_LEN_SIZE
                ..(DATA_LEN_SIZE
                    .checked_add(chunk_length as usize)
                    .expect("chunk size addition overflow"))],
        );

        let n = cmp::min(buf.len(), chunk.len());
        buf[..n].copy_from_slice(&chunk[..n]);
        self.recv_state.buffer.copy_from_slice(&chunk[n..]);

        Poll::Ready(Ok(n))

        // // Pin::new(&mut self.io).poll_read(cx, buf)
        // let n = ready!(Pin::new(&mut self.io).poll_read(cx, buf))?;
    }
}

async fn share_eph_pubkey<Io>(
    io: &mut Io,
    local_eph_pubkey: &EphemeralPublicKey,
    protocol_version: Version,
) -> Result<EphemeralPublicKey, Error>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    io.write_all(&protocol_version.encode_initial_handshake(local_eph_pubkey))
        .await?;

    let mut response_len = 0_u8;
    io.read_exact(slice::from_mut(&mut response_len)).await?;

    let mut buf = vec![0; response_len as usize];
    io.read_exact(&mut buf).await?;

    protocol_version.decode_initial_handshake(&buf)
}

async fn share_auth_signature<Io>(
    io: &mut Io,
    protocol_version: Version,
    pubkey: &VerificationKey,
    local_signature: &Signature,
) -> Result<proto::p2p::AuthSigMessage, Error>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    let buf = protocol_version.encode_auth_signature(pubkey, local_signature);
    io.write_all(&buf).await?;

    let mut buf = vec![0; protocol_version.auth_sig_msg_response_len()];
    io.read_exact(&mut buf).await?;

    protocol_version.decode_auth_signature(&buf)
}
