use crate::yubihsm::{create_hsm_connector, get_hsm_config};
use abscissa::Callable;
use signatory::ed25519;
use std::process;
use tendermint::public_keys::ConsensusKey;

/// The `yubihsm keys list` subcommand
#[derive(Debug, Default, Options)]
pub struct ListCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,
}

impl Callable for ListCommand {
    /// List all suitable Ed25519 keys in the HSM
    fn call(&self) {
        let hsm_connector = create_hsm_connector();

        let serial_number = hsm_connector.serial_number().unwrap_or_else(|e| {
            status_err!("couldn't get YubiHSM serial number: {}", e);
            process::exit(1);
        });

        let credentials = get_hsm_config().auth.credentials();

        let mut hsm = yubihsm::Client::open(hsm_connector, credentials, true).unwrap_or_else(|e| {
            status_err!("error connecting to YubiHSM2: {}", e);
            process::exit(1);
        });

        let objects = hsm.list_objects(&[]).unwrap_or_else(|e| {
            status_err!("couldn't list YubiHSM objects: {}", e);
            process::exit(1);
        });

        let mut keys = objects
            .iter()
            .filter(|o| o.object_type == yubihsm::object::Type::AsymmetricKey)
            .collect::<Vec<_>>();

        keys.sort_by(|k1, k2| k1.object_id.cmp(&k2.object_id));

        if keys.is_empty() {
            status_err!("no keys in this YubiHSM (#{})", serial_number);
            process::exit(0);
        }

        println!("Listing keys in YubiHSM #{}:", serial_number);

        for key in &keys {
            let public_key = hsm.get_public_key(key.object_id).unwrap_or_else(|e| {
                status_err!(
                    "couldn't get public key for asymmetric key #{}: {}",
                    key.object_id,
                    e
                );
                process::exit(1);
            });

            let key_id = format!("- #{}", key.object_id);

            // TODO: support for non-Ed25519 keys
            if public_key.algorithm == yubihsm::AsymmetricAlg::Ed25519 {
                status_attr_ok!(
                    key_id,
                    ConsensusKey::from(
                        ed25519::PublicKey::from_bytes(&public_key.as_ref()).unwrap()
                    )
                );
            } else {
                status_attr_err!(key_id, "unsupported algorithm: {:?}", public_key.algorithm);
            }
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(ListCommand);
