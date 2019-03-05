use abscissa::Callable;
use bip39::Mnemonic;
use chrono::{SecondsFormat, Utc};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand_os::{rand_core::RngCore, OsRng};
use sha2::Sha512;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process,
};
use subtle_encoding::{bech32::Bech32, hex};
use yubihsm::{
    authentication, object,
    setup::{Profile, Role},
    wrap, AuditOption, Capability, Connector, Credentials, Domain,
};
use zeroize::Zeroize;

/// Domain separation string used as "info" for HKDF
const HKDF_MNEMONIC_INFO: &[u8] = b"yubihsm setup BIP39 derivation";

/// Language used when generating the Mnemonic phrase
const BIP39_LANGUAGE: bip39::Language = bip39::Language::English;

/// Domain separation for derivation hierarchy versions (ala BIP43's "purpose" field)
const DERIVATION_VERSION: &[u8] = b"1";

/// Key size to use for generating passwords and wrap keys (256-bits).
/// This results in a 24-word BIP39 `Mnemonic` phrase.
const KEY_SIZE: usize = 32;

/// Role names
const ADMIN_ROLE_NAME: &str = "admin";
const OPERATOR_ROLE_NAME: &str = "operator";
const AUDITOR_ROLE_NAME: &str = "auditor";
const VALIDATOR_ROLE_NAME: &str = "validator";

/// The `yubihsm setup` subcommand: performs initial device provisioning
/// including creation of initial authentication and wrap keys.
#[derive(Debug, Default, Options)]
pub struct SetupCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Only print derived keys - do not reinitialize HSM
    #[options(short = "p", long = "print-only")]
    pub print_only: bool,

    /// Restore an HSM from an existing 24-word remonic
    #[options(short = "r", long = "restore")]
    pub restore: bool,

    /// Write a provisioning report as JSON to the given filename
    #[options(short = "w", long = "write-report")]
    pub write_report: Option<PathBuf>,
}

impl Callable for SetupCommand {
    /// Perform initial YubiHSM dervice provisioning
    fn call(&self) {
        let hsm_connector = crate::yubihsm::connector();
        let hsm_serial_number = get_hsm_client(&hsm_connector)
            .device_info()
            .expect("error getting device info")
            .serial_number;

        let mnemonic = if self.restore {
            println!("Restoring and reprovisioning YubiHSM from existing 24-word mnemonic phrase.");
            println!();

            read_mnemonic_from_stdin("*** Enter mnemonic (separate words with spaces): ")
        } else {
            generate_mnemonic_from_hsm_and_os_csprngs(&hsm_connector)
        };

        let roles = derive_roles_from_mnemonic(&mnemonic);
        let wrap_key = derive_wrap_key_from_mnemonic(&mnemonic, 1);

        // TODO(tarcieri): support for enabling forced auditing
        let profile = Profile::default()
            .audit_option(AuditOption::Off)
            .roles(roles)
            .wrap_keys(vec![wrap_key]);

        let operator_password = RolePassword::derive_from_mnemonic(&mnemonic, OPERATOR_ROLE_NAME);
        let auditor_password = RolePassword::derive_from_mnemonic(&mnemonic, AUDITOR_ROLE_NAME);
        let validator_password = RolePassword::derive_from_mnemonic(&mnemonic, VALIDATOR_ROLE_NAME);

        // Re-derive wrap key for display
        // TODO(tarcieri): allow access to the underlying wrap key secret in `yubihsm` crate to avoid this
        let mut wrapkey_hex = {
            let mut bytes =
                derive_secret_from_mnemonic(&mnemonic, &[b"wrap", serialize_key_id(1).as_bytes()]);

            let hex_str = String::from_utf8(hex::encode(bytes)).unwrap();
            bytes.zeroize();
            hex_str
        };

        if self.print_only {
            if self.restore {
                println!("Below are all keys/passwords derived from your 24-word admin mnemonic:");
            } else {
                println!(
                    "Below is a randomly generated 24-word admin mnemonic and derived keys/passwords:"
                );
            }
        } else {
            println!("This process will *ERASE* the configured YubiHSM2 and reinitialize it:");
            println!();
            println!("- YubiHSM serial: {}", hsm_serial_number);
            println!();
            println!("Authentication keys with the following IDs and passwords will be created:");
        }

        println!();
        println!("- key 0x0001: admin:");
        println!();
        print_mnemonic(&mnemonic);
        println!();
        println!(
            "- authkey 0x0002 [operator]:  {}",
            operator_password.as_str()
        );
        println!(
            "- authkey 0x0003 [auditor]:   {}",
            auditor_password.as_str()
        );
        println!(
            "- authkey 0x0004 [validator]: {}",
            validator_password.as_str()
        );

        println!("- wrapkey 0x0001 [primary]:   {}", &wrapkey_hex);
        wrapkey_hex.zeroize();

        if self.print_only {
            process::exit(0);
        }

        prompt_for_user_approval("Are you SURE you want erase and reinitialize this HSM?");

        let report = yubihsm::setup::erase_device_and_init_with_profile(
            hsm_connector.clone(),
            crate::yubihsm::config().auth.credentials(),
            profile,
        )
        .unwrap_or_else(|e| hsm_error(e.as_fail()));

        status_ok!(
            "Success",
            "reinitialized YubiHSM (serial: {})",
            hsm_serial_number
        );

        if let Some(ref report_path) = self.write_report {
            status_ok!(
                "Writing",
                "provisioning report to: {}",
                report_path.display()
            );

            let mut report_file = File::create(report_path).unwrap_or_else(|e| {
                panic!("couldn't create report file: {}", e);
            });

            report_file
                .write_all(report.to_json().as_bytes())
                .unwrap_or_else(|e| {
                    panic!("error writing report: {}", e);
                })
        }
    }
}

/// Read the mnemonic phrase from STDIN
fn read_mnemonic_from_stdin(prompt: &str) -> Mnemonic {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input_string = String::new();
    io::stdin()
        .read_line(&mut input_string)
        .expect("error reading mnemonic from STDIN!");

    let input_words: Vec<_> = input_string.split_whitespace().collect();
    let input_phrase = input_words.join(" ");
    input_string.zeroize();

    let result = Mnemonic::from_phrase(input_phrase, BIP39_LANGUAGE).unwrap_or_else(|e| {
        eprintln!("*** ERROR: Couldn't decode mnemonic: {}", e);
        process::exit(1);
    });

    println!("\nMnemonic phrase decoded/checksummed successfully!\n");

    result
}

/// Display the mnemonic as two groups of 12 words
fn print_mnemonic(mnemonic: &Mnemonic) {
    let words: Vec<&str> = mnemonic.phrase().split(' ').collect();
    let words_len = words.len();

    for word_group in &[&words[..(words_len / 2)], &words[(words_len / 2)..]] {
        let mut word_group_joined = word_group.join(" ");
        println!("    {}", word_group_joined);
        word_group_joined.zeroize();
    }
}

/// Get an HSM client from the provided connector
///
/// We need to create our own client here since the global one maintains
/// a persistent connection, and we need to close this one before we can
/// reprovision the HSM
fn get_hsm_client(hsm_connector: &Connector) -> yubihsm::Client {
    yubihsm::Client::open(
        hsm_connector.clone(),
        crate::yubihsm::config().auth.credentials(),
        false,
    )
    .unwrap_or_else(|e| hsm_error(&e))
}

/// Generate entropy by combining entropy both from the host OS and from the
/// YubiHSM2 itself (which includes an internal CSPRNG).
///
/// These are both used as input key material (IKM) for a key derivation
/// function (HKDF) in order to derive the recovery passphrase, which ideally
/// ensures that the passphrase will be securely random so long as at least
/// one of the two inputs is secure.
fn generate_mnemonic_from_hsm_and_os_csprngs(hsm_connector: &Connector) -> Mnemonic {
    let hsm_client = get_hsm_client(hsm_connector);

    // Obtain half of the IKM from the YubiHSM
    let mut ikm = hsm_client
        .get_pseudo_random(KEY_SIZE / 2)
        .unwrap_or_else(|e| hsm_error(&e));

    // Obtain another half of the IKM from the host OS
    ikm.extend_from_slice(&[0u8; KEY_SIZE / 2]);
    OsRng::new().unwrap().fill_bytes(&mut ikm[(KEY_SIZE / 2)..]);

    let kdf = Hkdf::<Sha512>::extract(None, &ikm);

    // 32-bytes (256-bits) -> 24 BIP32 words
    let mut okm = [0u8; KEY_SIZE];
    kdf.expand(HKDF_MNEMONIC_INFO, &mut okm).unwrap();
    ikm.zeroize();

    let result = Mnemonic::from_entropy(&okm, BIP39_LANGUAGE).unwrap();
    okm.zeroize();

    result
}

/// Derive the default roles form the given BIP39 `Mnemonic`
fn derive_roles_from_mnemonic(mnemonic: &Mnemonic) -> Vec<Role> {
    let admin_role = derive_admin_role_from_mnemonic(mnemonic);

    // operator
    let operator_role = derive_role_from_mnemonic(mnemonic, 2, OPERATOR_ROLE_NAME)
        .capabilities(
            Capability::GENERATE_ASYMMETRIC_KEY
                | Capability::PUT_ASYMMETRIC_KEY
                | Capability::GENERATE_HMAC_KEY
                | Capability::PUT_HMAC_KEY
                | Capability::IMPORT_WRAPPED
                | Capability::EXPORT_WRAPPED
                | Capability::GET_OPAQUE
                | Capability::GET_OPTION
                | Capability::GET_LOG_ENTRIES
                | Capability::SIGN_ATTESTATION_CERTIFICATE,
        )
        .delegated_capabilities(Capability::all())
        .domains(Domain::all());

    // auditor
    let auditor_role = derive_role_from_mnemonic(mnemonic, 3, AUDITOR_ROLE_NAME)
        .capabilities(
            Capability::GET_LOG_ENTRIES
                | Capability::GET_OPTION
                | Capability::PUT_OPTION
                | Capability::GET_OPAQUE,
        )
        .delegated_capabilities(Capability::empty())
        .domains(Domain::all());

    // validator
    let validator_role = derive_role_from_mnemonic(mnemonic, 4, VALIDATOR_ROLE_NAME)
        .capabilities(
            Capability::SIGN_ECDSA
                | Capability::SIGN_EDDSA
                | Capability::SIGN_ATTESTATION_CERTIFICATE
                | Capability::GET_LOG_ENTRIES,
        )
        .delegated_capabilities(Capability::empty())
        .domains(Domain::DOM1);

    vec![admin_role, operator_role, auditor_role, validator_role]
}

/// Derive the admin role from the given mnemonic.
///
/// The admin role is somewhat different from the others and confers total
/// authority over the HSM device.
fn derive_admin_role_from_mnemonic(mnemonic: &Mnemonic) -> Role {
    let admin_credentials = Credentials::new(
        1,
        authentication::Key::derive_from_password(mnemonic.as_ref().as_bytes()),
    );

    Role::new(admin_credentials)
        .authentication_key_label(create_object_label(ADMIN_ROLE_NAME))
        .capabilities(Capability::all())
        .delegated_capabilities(Capability::all())
        .domains(Domain::all())
}

/// Derive the initial settings for a role from the given mnemonic
fn derive_role_from_mnemonic(mnemonic: &Mnemonic, key_id: object::Id, role_name: &str) -> Role {
    let role_password = RolePassword::derive_from_mnemonic(&mnemonic, role_name);

    let role_credentials = Credentials::new(
        key_id,
        authentication::Key::derive_from_password(role_password.as_bytes()),
    );

    Role::new(role_credentials).authentication_key_label(create_object_label(role_name))
}

/// Passwords for a given role, derived from a BIP39 `Mnemonic`.
/// These are serialized as Bech32 for compactness.
struct RolePassword(String);

impl RolePassword {
    /// Derive a role password from the given BIP39 `Mnemonic`
    pub fn derive_from_mnemonic(mnemonic: &Mnemonic, role_name: &str) -> Self {
        let mut secret_key =
            derive_secret_from_mnemonic(mnemonic, &[b"role", role_name.as_bytes()]);

        // YubiHSM authentication keys are 2 x AES-128 keys, derived using
        // PBKDF2. This means we derive no value from more than 128-bits of
        // entropy in the password, whereas shorter passwords are more
        // convenient.
        //
        // For that reason, truncate the derived secret to 16-bytes
        let truncated_secret_key = &secret_key[..(KEY_SIZE / 2)];

        let result = RolePassword(
            Bech32::default().encode(format!("kms-{}-password-", role_name), truncated_secret_key),
        );
        secret_key.zeroize();

        result
    }

    /// Borrow the raw password as a `&str`
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Borrow the raw password as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Drop for RolePassword {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Derive a wrap key from the given BIP39 `Mnemonic`
fn derive_wrap_key_from_mnemonic(mnemonic: &Mnemonic, key_id: object::Id) -> wrap::Key {
    // Capabilities given to the initial wrap key:
    // wrap/unwrap both data and objects
    let wrap_key_capabilities = Capability::EXPORT_WRAPPED
        | Capability::IMPORT_WRAPPED
        | Capability::WRAP_DATA
        | Capability::UNWRAP_DATA;

    // Allow the initial wrap key to create new objects with any other
    // capabilities.
    //
    // To prevent escalation of privilege, no roles except administrators
    // will be given the capability to export other objects.
    let wrap_key_delegated_capabilities = Capability::all();

    // Make the wrap key accessible from all domains. This allows it to
    // import and export objects from any domain.
    let wrap_key_domains = Domain::all();

    // Label to put on the wrap key
    let wrap_key_label = create_object_label("primary");

    // Includes the key ID in the derivation path, which allows us to derive
    // other wrap keys from the same seed `Mnemonic` phrase in the event one
    // has been compromised.
    wrap::Key::from_bytes(
        key_id,
        &derive_secret_from_mnemonic(mnemonic, &[b"wrap", serialize_key_id(key_id).as_bytes()]),
    )
    .unwrap()
    .label(wrap_key_label)
    .capabilities(wrap_key_capabilities)
    .delegated_capabilities(wrap_key_delegated_capabilities)
    .domains(wrap_key_domains)
}

/// Serialize a key ID as bytes for use in a derivation path
fn serialize_key_id(key_id: object::Id) -> String {
    format!("0x{:04x}", key_id)
}

/// Derive secrets from the given BIP39 `Mnemonic` ala a BIP32 (hardened)
/// derivation hierarchy.
fn derive_secret_from_mnemonic(mnemonic: &Mnemonic, path: &[&[u8]]) -> [u8; KEY_SIZE] {
    debug!(
        "deriving secret for path: /{}",
        path.iter()
            .map(|component| String::from_utf8_lossy(component))
            .collect::<Vec<_>>()
            .join("/")
    );

    // Domain separate the toplevel of the derivation hierarchy ala BIP43
    let mut seed_hmac = Hmac::<Sha512>::new_varkey(mnemonic.entropy()).unwrap();
    seed_hmac.input(DERIVATION_VERSION);

    let mut seed = [0u8; KEY_SIZE];
    seed.copy_from_slice(&seed_hmac.result().code()[KEY_SIZE..]);

    // Simplified BIP32-like hierarchical derivation
    path.iter()
        .enumerate()
        .fold(seed, |mut parent_key, (i, elem)| {
            let mut hmac = Hmac::<Sha512>::new_varkey(&parent_key).unwrap();
            hmac.input(elem);

            let hmac_result = hmac.result().code();
            parent_key.zeroize();

            let (secret_key, chain_code) = hmac_result.split_at(KEY_SIZE);
            let mut child_key = [0u8; KEY_SIZE];

            if i < path.len() - 1 {
                // Use chain code for all but the last element
                child_key.copy_from_slice(chain_code);
            } else {
                // Use secret key for the last element
                child_key.copy_from_slice(secret_key);
            }

            child_key
        })
}

/// Create a label for a newly generated object which tags it with the date
/// it was created
fn create_object_label(label_prefix: &str) -> object::Label {
    // e.g. 2019-02-26T18:03:53Z
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    object::Label::from([label_prefix, &timestamp].join(":").as_ref())
}

/// Prompt the user to ensure they want to proceed
fn prompt_for_user_approval(prompt: &str) {
    print!("\n*** {} (y/N): ", prompt);
    io::stdout().flush().unwrap();

    let mut choice_in = String::new();
    io::stdin()
        .read_line(&mut choice_in)
        .expect("Failed to read user input");

    let choice = choice_in.trim();

    if choice != "y" && choice != "Y" {
        println!("Aborting");
        process::exit(1);
    }
}

/// Handler for HSM errors
fn hsm_error(e: &failure::Fail) -> ! {
    status_err!("HSM error: {}", e);

    // TODO: handle exits via abscissa
    process::exit(1);
}
