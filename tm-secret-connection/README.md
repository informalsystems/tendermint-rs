# Tendermint SecretConnection

Transport layer encryption for Tendermint P2P connections.

## Synopsis

`SecretConnection` is a modern implementation of the [Station-to-Station protocol][sts]
used for encrypting connections between nodes in Tendermint-based networks.

It uses the following algorithms:

- **Key Agreement**: [X25519]
- **Digital Signatures**: [Ed25519]
- **Symmetric Encryption**: [ChaCha20Poly1305]

[sts]: https://en.wikipedia.org/wiki/Station-to-Station_protocol
[X25519]: https://tools.ietf.org/html/rfc7748#section-5
[Ed25519]: https://tools.ietf.org/html/rfc8032
[ChaCha20Poly1305]: https://tools.ietf.org/html/rfc7539
