use std::sync::Mutex;

use signatory_ledger_cosval::Ed25519CosmosAppSigner;

lazy_static! {
    static ref HSM_CLIENT: Mutex<Ed25519CosmosAppSigner> = Mutex::new(create_hsm_client());
}

// pub fn get_hsm_client() -> MutexGuard<'static, Ed25519CosmosAppSigner> {
//     HSM_CLIENT.lock().unwrap()
// }

fn create_hsm_client() -> Ed25519CosmosAppSigner {
    Ed25519CosmosAppSigner::connect().unwrap()
}