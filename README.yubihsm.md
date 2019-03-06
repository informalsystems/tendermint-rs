# YubiHSM 2 + Tendermint KMS

The [YubiHSM 2] from Yubico is a relatively low-cost solution for online
key storage featuring support for random key generation, encrypted backup
and export, and audit logging.

This document describes how to configure a YubiHSM 2 for production use
with Tendermint KMS.

## Compiling `tmkms` with YubiHSM support

Please see the [toplevel README.md] for prerequisites for compiling `tmkms`
from source code. You will need: Rust (stable, 1.31+), a C compiler,
`pkg-config`, and `libusb` (1.0+) installed.

There are two ways to install `tmkms` with YubiHSM 2 support, and in either
case, you will need to pass the `--features=yubihsm` parameter to enable
YubiHSM 2 support:

### Compiling from source code (via git)

`tmkms` can be compiled directly from the git repository source code using the
following method.

```
$ git clone https://github.com/tendermint/kms.git && cd kms
[...]
$ cargo build --release --features=yubihsm
```

If successful, this will produce a `tmkms` executable located at
`./target/release/tmkms`

### Installing with the `cargo install` command

With Rust (1.31+) installed, you can install tmkms with the following:

```
cargo install tmkms --features=yubihsm
```

Or to install a specific version (recommended):

```
cargo install tmkms --features=yubihsm --version=0.4.0
```

This command installs `tmkms` directly from packages hosted on Rust's
[crates.io] service. Package authenticity is verified via the
[crates.io index] (itself a git repository) and by SHA-256 digests of
released artifacts.

However, if newer dependencies are available, it may use newer versions
besides the ones which are "locked" in the source code repository. We
cannot verify those dependencies do not contain malicious code. If you would
like to ensure the dependencies in use are identical to the main repository,
please build from source code instead.

### Verifying YubiHSM support was included in a build

Run the following subcommand of the resulting `tmkms` executable to ensure
that YubiHSM 2 support was compiled-in successfully:

```
$ tmkms yubihsm help                                                                           127 ↵
tmkms 0.4.0
Tony Arcieri <tony@iqlusion.io>, Ismail Khoffi <Ismail.Khoffi@gmail.com>
Tendermint Key Management System

USAGE:
  tmkms <SUBCOMMAND>

FLAGS:
  -h, --help     Prints help information
  -V, --version  Prints version information

SUBCOMMANDS:
  detect  detect all YubiHSM2 devices connected via USB
  help    show help for the 'yubihsm' subcommand
  keys    key management subcommands
  setup   initial device setup and configuration
  test    perform a signing test
```

If `detect`, `help`, `keys`, `setup` etc are listed under `SUBCOMMANDS` then
the build was successfully configured with YubiHSM 2 support.

## udev configuration

On Linux, you will need to grant `tmkms` access to the YubiHSM 2 using
rules for the udev subsystem. Otherwise, you'll get an error like this:

```
$ tmkms yubihsm detect
error: couldn't detect USB devices: USB error: USB(bus=1,addr=4):
       error opening device: Access denied (insufficient permissions)
```

You'll need to create a POSIX group, e.g. `yubihsm` which is allowed to
access the YubiHSM2, and then add the following rules file under the
`/etc/udev/rules.d` directory, e.g. `/etc/udev/rules.d/10-yubihsm.rules`:

```
SUBSYSTEMS=="usb", ATTRS{product}=="YubiHSM", GROUP=="yubihsm"
```

Note that creating this file does not have an immediate effect: you'll
need to reload the udev subsystem, either by rebooting or running the
following command:

```
$ udevadm control --reload-rules && udevadm trigger
```

For the rules above to apply, make sure you run `tmkms` as a user which is a
member of the `yubihsm` group!

## Production YubiHSM 2 setup

`tmkms` contains built-in support for fully automated production YubiHSM 2
setup, including deterministically generating authentication keys and backup
encryption keys from a [BIP39] 24-word seed phrase. This allows new YubiHSM 2s
to be provisioned with the same initial set of keys from the phrase alone,
while also creating multiple "roles" within the HSM.

Alternatively Yubico provides the [yubihsm-setup] tool, however the setup
process internal to `tmkms` provides a "happy path" for Tendermint validator
usage, and also operational processes which should be familiar to
cryptocurrency users.

### Configuring `tmkms` for initial setup

In order to perform setup, `tmkms` needs a minimal configuration file which
contains the credentials needed to authenticate to the HSM with an
administrator key.

This configuration should be placed in a file called: `tmkms.toml`.
By default `tmkms` will look for this file in the current working directory,
however most subcommands take a `-c /path/to/tmkms.toml` argument if you
would like to place it somewhere else.

Here is an example `tmkms.toml` file which can be used for initial setup
with a YubiHSM 2 which is still configured with its default authentication
keys (i.e. authentication key 1, with a default password of `password`):

```toml
[[providers.yubihsm]]
adapter = { type = "usb" }
auth = { key = 1, password = "password" }
```

If you have changed the default authentication key ID and/or password, you
will need to provide the correct credentials.

NOTE: if you have *lost or forgotten* the admin authentication key, you
can *factory reset* the YubiHSM 2 to a default state (wiping all keys)
by pushing down on the top (LED) immediately after inserting it and continuing
to push down on it for 10 seconds.

### `tmkms yubihsm setup`: Initial YubiHSM setup

**WARNING: THIS PROCESS PERFORMS A FACTORY RESET OF THE YUBIHSM, DELETING ALL
EXISTING KEYS AND REPLACING THEM WITH NEW ONES. MAKE SURE YOU HAVE MADE BACKUPS
OF IMPORTANT KEYS BEFORE PROCEEDING!!!**

After configuring your YubiHSM 2's credentials in `tmkms.toml`, you can run the
following command to perform automatic setup:

```
$ tmkms yubihsm setup
```

We recommend this process be performed on an airgapped computer which is not
connected to any network. It will generate master secrets which can be used
to decrypt encrypted backups of keys within the HSM.

This process will perform the following steps:

- Generate a random 24-word recovery mnemonic phrase from randomness taken
  from the host OS as well as the YubiHSM2 itself.
- Deterministically (ala BIP32) generate authentication keys for the following
  four roles within the YubiHSM (with [yubihsm-shell] compatible passwords):
  - **admin** (authentication key `0x0001`): full access to all HSM capabilities
  - **operator** (authentication key `0x0002`): ability to generate new signing
    keys, export/import encrypted backups of keys, and view the audit log
  - **auditor** (authentication key `0x0003`): ability to view and consume the
    audit log
  - **validator** (authentication key `0x0004`): ability to generate signatures
    using signing keys loaded within the device
- Deterministically generate "wrap key" `0x0001`: symmetric encryption key
  (AES-CCM) used for making encrypted backups of other keys generated within
  the device. If you have existing keys you would like to transfer out of other
  YubiHSM 2s, this key can be imported into those HSMs in order to export
  encrypted backups (see below).

Notably different from cryptocurrency hardware wallets: this process does not
actually generate any signing keys, only authentication keys and the wrap key.
Generating validator signing keys (and creating backups) occurs in a subsequent
step (see below).

The following is example output from running the above command:

```
$ tmkms yubihsm setup
This process will *ERASE* the configured YubiHSM2 and reinitialize it:

- YubiHSM serial: 9876543210

Authentication keys with the following IDs and passwords will be created:

- key 0x0001: admin:

    double section release consider diet pilot flip shell mother alone what fantasy
    much answer lottery crew nut reopen stereo square popular addict just animal

- authkey 0x0002 [operator]:  kms-operator-password-1k02vtxh4ggxct5tngncc33rk9yy5yjhk
- authkey 0x0003 [auditor]:   kms-auditor-password-1s0ynq69ezavnqgq84p0rkhxvkqm54ks9
- authkey 0x0004 [validator]: kms-validator-password-1x4anf3n8vqkzm0klrwljhcx72sankcw0
- wrapkey 0x0001 [primary]:   21a6ca8cfd5dbe9c26320b5c4935ff1e63b9ab54e2dfe24f66677aba8852be13

*** Are you SURE you want erase and reinitialize this HSM? (y/N):
```

NOTE: the admin password is *displayed* on two separate lines. When using it
from [yubihsm-shell] or as the `password` field in tmkms.toml, it is not split
across multiple lines and is separated only by a single space between words.

If you are certain you are ready to initialize your first YubiHSM 2, type `y`
to proceed:

```
*** Are you SURE you want erase and reinitialize this HSM? (y/N): y
21:08:09 [WARN] factory resetting HSM device! all data will be lost!
21:08:10 [INFO] waiting for device reset to complete
21:08:11 [INFO] installed temporary setup authentication key into slot 65534
21:08:11 [WARN] deleting default authentication key from slot 1
21:08:11 [INFO] installing role: admin:2019-03-05T20:31:07Z
21:08:11 [INFO] installing role: operator:2019-03-05T20:31:08Z
21:08:11 [INFO] installing role: auditor:2019-03-05T20:31:08Z
21:08:11 [INFO] installing role: validator:2019-03-05T20:31:08Z
21:08:11 [INFO] installing wrap key: primary:2019-03-05T20:31:08Z
21:08:11 [INFO] storing provisioning report in opaque object 0xfffe
21:08:11 [WARN] deleting temporary setup authentication key from slot 65534
     Success reinitialized YubiHSM (serial: 9876543210)
```

Make sure to write down the 24-word recovery phrase and store it in a
secure location!

### Initializing additional HSMs from an existing 24-word recovery phrase

After initializing your first HSM, you can bootstrap additional YubiHSM 2s as
a clone of the initial one using the same 24-word recovery phrase. To do that,
pass the `-r` (or `--restore`) flag when running the setup command:

```
$ tmkms yubihsm setup -r
Restoring and reprovisioning YubiHSM from existing 24-word mnemonic phrase.

*** Enter mnemonic (separate words with spaces): double section release consider [...]

Mnemonic phrase decoded/checksummed successfully!

This process will *ERASE* the configured YubiHSM2 and reinitialize it:

- YubiHSM serial: 9876543211

Authentication keys with the following IDs and passwords will be created:

- key 0x0001: admin:

    double section release consider diet pilot flip shell mother alone what fantasy
    much answer lottery crew nut reopen stereo square popular addict just animal

- authkey 0x0002 [operator]:  kms-operator-password-1k02vtxh4ggxct5tngncc33rk9yy5yjhk
- authkey 0x0003 [auditor]:   kms-auditor-password-1s0ynq69ezavnqgq84p0rkhxvkqm54ks9
- authkey 0x0004 [validator]: kms-validator-password-1x4anf3n8vqkzm0klrwljhcx72sankcw0
- wrapkey 0x0001 [primary]:   21a6ca8cfd5dbe9c26320b5c4935ff1e63b9ab54e2dfe24f66677aba8852be13

*** Are you SURE you want erase and reinitialize this HSM? (y/N): y
21:47:18 [WARN] factory resetting HSM device! all data will be lost!
21:47:19 [INFO] waiting for device reset to complete
21:47:21 [INFO] installed temporary setup authentication key into slot 65534
21:47:21 [WARN] deleting default authentication key from slot 1
21:47:21 [INFO] installing role: admin:2019-03-05T21:47:02Z
21:47:21 [INFO] installing role: operator:2019-03-05T21:47:03Z
21:47:21 [INFO] installing role: auditor:2019-03-05T21:47:03Z
21:47:21 [INFO] installing role: validator:2019-03-05T21:47:03Z
21:47:21 [INFO] installing wrap key: primary:2019-03-05T21:47:03Z
21:47:21 [INFO] storing provisioning report in opaque object 0xfffe
21:47:21 [WARN] deleting temporary setup authentication key from slot 65534
     Success reinitialized YubiHSM (serial: 9876543211)
```

## `tmkms yubihsm keys generate`: signing key generation

The `tmkms` YubiHSM backend is designed to support signing keys which are
randomly generated by the device's internal cryptographically secure random
number generator, as opposed to ones which are deterministically generated
from the 24-word BIP39 mnemonic phrase.

This means you will need to do the following:

- Run `tmkms yubihsm keys generate` to create signing keys
  (i.e. validator consensus keys)
- Retain backups of these keys for disaster recovery

This command integrates a feature to export a backup of the keys at the
time they are generated, which is compatible with the [yubihsm-shell] tool.

Below is an example of the command to generate and export an encrypted backup
of an Ed25519 signing key:

```
$ tmkms yubihsm keys generate 1 -l "steakz4u-validator:2019-03-06T01:25:39Z" -b steakz4u-validator-key.enc
 Generated key #1: cosmosvalconspub1zcjduepqtvzxa733n7dhrjf247n0jtdwsvvsd4jgqvzexj5tkwerpzy5sugsvmfja3
     Wrote backup of key 1 (encrypted under wrap key 1) to steakz4u-validator-key.enc
```

This operation must be performed after configuring `tmkms.toml` with one of
the following sets of credentials:

- The `admin` key and associated 24-word password (i.e. key ID `0x0001`)
- The `operator` key (`0x0002`) and associated `kms-operator-password`

### Parameters

- `tmkms yubihsm keys generate 1` - generates asymmetric key 0x0001, which is by
  default an Ed25519 signing key.
- `-l` (or `--label`): an up-to-40-character label describing the key
- `-b` (or `--backup`): path to a file where an *encrypted* backup of the
  generated key should be written
- (not used in the example) `-w` (or `--wrapkey`): ID of the "wrap"
  (i.e encryption) key used to encrypt the backup key. It defaults to wrap
  key 0x0001 which was automatically generated as part of the
  `tmkms yubihsm setup` process.

## `tmkms yubihsm keys list`: list signing keys

The following command lists keys in the HSM:

```
$ tmkms yubihsm keys list
Listing keys in YubiHSM #9876543211:
- #1:	 cosmosvalconspub1zcjduepqtvzxa733n7dhrjf247n0jtdwsvvsd4jgqvzexj5tkwerpzy5sugsvmfja3
```

## Exporting and Importing Keys

`tmkms` contains functionality for exporting and importing keys, including
making encrypted backups of keys, and also importing existing
`priv_validator.json` keys.

We recommend you randomly generate keys using the above
`tmkms yubihsm keys generate` procedure to avoid exposing plaintext copies
of signing private keys outside of the HSM. However, below are instructions
which can hopefully accommodate any situation you happen to be in with regard
to exporting and importing existing keys.

### `tmkms yubihsm keys export`: export encrypted backups of signing keys

If you ran `tmkms yubihsm keys generate` (or equivalent [yubihsm-shell])
command without creating a backup, the `keys export` subcommand can also 
export a backup:

```
$ yubihsm keys export --id 1 steakz4u2-validator-key.enc
  Exported key 0x0001 (encrypted under wrap key 0x0001) to steakz4u2-validator-key.enc
```

The backups generated are compatible with the ones generated by the Yubico
`yubihsm-shell` utility.

#### Parameters

- `-i` (or `--id`): ID of the asymmetric key to export
- `-w` (or `--wrapkey`): ID of the wrap key under which the exported key will
  be encrypted.

### `tmkms yubihsm keys import`:  import encrypted backups of signing keys 

After generating a key on a YubiHSM and exporting a backup, you can import the
encrypted copy into another HSM with the following command:

```
$ tmkms yubihsm keys import steakz4u-validator-key.enc
    Imported key 0x0001: cosmosvalconspub1zcjduepqtvzxa733n7dhrjf247n0jtdwsvvsd4jgqvzexj5tkwerpzy5sugsvmfja3
```

### Exporting keys from previously configured YubiHSM 2s

If you've previously configured a production key within a YubiHSM 2 and wish to
securely export it and import it into a YubiHSM 2 provisioned using the
`tmkms yubihsm setup` workflow, here are the steps to securely export it.

#### Note wrap key during the `tmkms yubihsm setup` procedure

Among the keys generated during the initial procedure is the wrap key, which
is the encryption key used for all backups. It's at the bottom of this list:

```
- authkey 0x0002 [operator]:  kms-operator-password-1k02vtxh4ggxct5tngncc33rk9yy5yjhk
- authkey 0x0003 [auditor]:   kms-auditor-password-1s0ynq69ezavnqgq84p0rkhxvkqm54ks9
- authkey 0x0004 [validator]: kms-validator-password-1x4anf3n8vqkzm0klrwljhcx72sankcw0
- wrapkey 0x0001 [primary]:   21a6ca8cfd5dbe9c26320b5c4935ff1e63b9ab54e2dfe24f66677aba8852be13
```

(i.e. `wrapkey 0x0001 [primary]`)

The number `21a6ca8cfd5dbe9c26320b5c4935ff1e63b9ab54e2dfe24f66677aba8852be13`
is the hex serialization of an AES-256-CCM encryption key, and also compatible
with the syntax used by the [yubihsm-shell] utility.

If you can authenticate to a YubiHSM 2 which contains an existing key you with
to export, you can import the wrap key to export it under into the HSM with
the following `yubihsm-shell` command:

```
yubihsm> put wrapkey 0 1 wrapkey 1 export-wrapped,import-wrapped exportable-under-wrap,sign-ecdsa,sign-eddsa 21a6ca8cfd5dbe9c26320b5c4935ff1e63b9ab54e2dfe24f66677aba8852be13
Stored Wrap key 0x0001
```

#### Parameters

- `put wrapkey 0 1 wrapkey`: put the specified wrap key (via session 0) into slot `0x0001`
- `1`: put the wrap key into [domain] 1 (use any of the domains the original key
  is accessible from)
- `export-wrapped,import-wrapped`: grant the [capabilities] to export and
  import other keys
- `exportable-under-wrap,sign-ecdsa,sign-eddsa`: [delegated capabilities] which
  allow imported keys to be exported again, as well as used to generate ECDSA
  and "EdDSA" (i.e. Ed25519) signatures.
- `[hex string]`: raw AES-256-CCM wrap key to import

[YubiHSM 2]: https://www.yubico.com/product/yubihsm-2/
[toplevel README.md]: https://github.com/tendermint/kms/blob/master/README.md#installation
[crates.io]: https://crates.io
[crates.io index]: https://github.com/rust-lang/crates.io-index
[BIP39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
[yubihsm-setup]: https://developers.yubico.com/YubiHSM2/Component_Reference/yubihsm-setup/
[yubihsm-shell]: https://developers.yubico.com/YubiHSM2/Component_Reference/yubihsm-shell/
[domain]: https://developers.yubico.com/YubiHSM2/Concepts/Domain.html
[capabilities]: https://developers.yubico.com/YubiHSM2/Concepts/Capability.html
[delegated capabilities]: https://developers.yubico.com/YubiHSM2/Concepts/Effective_Capabilities.html
