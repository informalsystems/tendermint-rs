- `[tendermint]` Add the following impls for `ed25519-consensus`:
  * `From<ed25519_consensus::SigningKey` for `tendermint::PrivateKey`
  * `From<ed25519_consensus::SigningKey>` for `tendermint::SigningKey`
  * `From<ed25519_consensus::VerificationKey>` for `tendermint::PublicKey`
  * `From<ed25519_consensus::VerificationKey>` for `tendermint::VerificationKey`
  ([\#1401](https://github.com/informalsystems/tendermint-rs/pull/1401))
