use subtle_encoding::hex;

use tendermint_p2p::secret_connection::PublicKey;

const EXAMPLE_SECRET_CONN_KEY: &str =
    "F7FEB0B5BA0760B2C58893E329475D1EA81781DD636E37144B6D599AD38AA825";

#[test]
fn test_secret_connection_pubkey_serialization() {
    let example_key =
        PublicKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_SECRET_CONN_KEY).unwrap()).unwrap();

    assert_eq!(
        example_key.to_string(),
        "117c95c4fd7e636c38d303493302d2c271a39669"
    );
}
