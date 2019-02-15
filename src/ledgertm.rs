use signatory_ledger_tm::Ed25519LedgerTmAppSigner;
use std::sync::Mutex;

// This instance is only used by CLI commands or tests
lazy_static! {
    static ref HSM_CLIENT: Mutex<Ed25519LedgerTmAppSigner> = Mutex::new(create_hsm_client());
}

fn create_hsm_client() -> Ed25519LedgerTmAppSigner {
    Ed25519LedgerTmAppSigner::connect().unwrap()
}
