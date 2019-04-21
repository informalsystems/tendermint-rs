//! `SecretConnection` integration tests

#[cfg(features = "secret-connection")]
use self::subtle_encoding::hex;
#[cfg(features = "secret-connection")]
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};
#[cfg(features = "secret-connection")]
use tendermint::secret_connection::Kdf;

#[cfg(features = "secret-connection")]
#[test]
fn test_derive_secrets_and_challenge_golden_test_vectors() {
    let f = File::open("tests/support/TestDeriveSecretsAndChallenge.golden").unwrap();
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let params: Vec<&str> = l.split(',').collect();

        let rand_secret_vector: Vec<u8> = hex::decode(params.get(0).unwrap()).unwrap();
        let mut rand_secret: [u8; 32] = [0x0; 32];
        rand_secret.copy_from_slice(&rand_secret_vector);

        let loc_is_least = bool::from_str(params.get(1).unwrap()).unwrap();
        let expected_recv_secret = hex::decode(params.get(2).unwrap()).unwrap();
        let expected_send_secret = hex::decode(params.get(3).unwrap()).unwrap();
        let expected_challenge = hex::decode(params.get(4).unwrap()).unwrap();
        let kdf = Kdf::derive_secrets_and_challenge(&rand_secret, loc_is_least);

        assert_eq!(
            expected_recv_secret, kdf.recv_secret,
            "Recv Secrets aren't equal"
        );
        assert_eq!(
            expected_send_secret, kdf.send_secret,
            "Send Secrets aren't equal"
        );
        assert_eq!(expected_challenge, kdf.challenge, "challenges aren't equal");
    }
}
