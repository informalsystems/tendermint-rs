use std::sync::Mutex;

use signatory_ledger_tm::Ed25519LedgerTmAppSigner;

lazy_static! {
    static ref HSM_CLIENT: Mutex<Ed25519LedgerTmAppSigner> = Mutex::new(create_hsm_client());
}

// pub fn get_hsm_client() -> MutexGuard<'static, Ed25519CosmosAppSigner> {
//     HSM_CLIENT.lock().unwrap()
// }

fn create_hsm_client() -> Ed25519LedgerTmAppSigner {
    Ed25519LedgerTmAppSigner::connect().unwrap()
}
