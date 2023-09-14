//! Tendermint validators

use serde::{Deserialize, Serialize};
use tendermint_proto::v0_38::types::{
    SimpleValidator as RawSimpleValidator, ValidatorSet as RawValidatorSet,
};
use tendermint_proto::Protobuf;

use crate::{
    account,
    crypto::signature::Verifier,
    crypto::Sha256,
    hash::Hash,
    merkle::{self, MerkleHash},
    prelude::*,
    public_key::deserialize_public_key,
    vote, Error, PublicKey, Signature,
};

/// Validator set contains a vector of validators
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawValidatorSet")]
pub struct Set {
    validators: Vec<Info>,
    proposer: Option<Info>,
    total_voting_power: vote::Power,
}

impl Set {
    pub const MAX_TOTAL_VOTING_POWER: u64 = (i64::MAX / 8) as u64;

    /// Constructor
    pub fn new(validators: Vec<Info>, proposer: Option<Info>) -> Set {
        Self::try_from_parts(validators, proposer, 0).unwrap()
    }

    fn try_from_parts(
        mut validators: Vec<Info>,
        proposer: Option<Info>,
        unvalidated_total_voting_power: i64,
    ) -> Result<Set, Error> {
        // Compute the total voting power
        let total_voting_power = validators
            .iter()
            .map(|v| v.power.value())
            .fold(0u64, |acc, v| acc.saturating_add(v));

        if total_voting_power > Self::MAX_TOTAL_VOTING_POWER {
            return Err(Error::total_voting_power_overflow());
        }

        // The conversion cannot fail as we have validated against a smaller limit.
        let total_voting_power: vote::Power = total_voting_power.try_into().unwrap();

        // If the given total voting power is not the default value,
        // validate it against the sum of voting powers of the participants.
        if unvalidated_total_voting_power != 0 {
            let given_val: vote::Power = unvalidated_total_voting_power.try_into()?;
            if given_val != total_voting_power {
                return Err(Error::total_voting_power_mismatch());
            }
        }

        Self::sort_validators(&mut validators);

        Ok(Set {
            validators,
            proposer,
            total_voting_power,
        })
    }

    /// Convenience constructor for cases where there is no proposer
    pub fn without_proposer(validators: Vec<Info>) -> Set {
        Self::new(validators, None)
    }

    /// Convenience constructor for cases where there is a proposer
    pub fn with_proposer(
        validators: Vec<Info>,
        proposer_address: account::Id,
    ) -> Result<Self, Error> {
        // Get the proposer.
        let proposer = validators
            .iter()
            .find(|v| v.address == proposer_address)
            .cloned()
            .ok_or_else(|| Error::proposer_not_found(proposer_address))?;

        // Create the validator set with the given proposer.
        // This is required by IBC on-chain validation.
        Ok(Self::new(validators, Some(proposer)))
    }

    /// Get Info of the underlying validators.
    pub fn validators(&self) -> &Vec<Info> {
        &self.validators
    }

    /// Get proposer
    pub fn proposer(&self) -> &Option<Info> {
        &self.proposer
    }

    /// Get total voting power
    pub fn total_voting_power(&self) -> vote::Power {
        self.total_voting_power
    }

    /// Sort the validators according to the current Tendermint requirements
    /// (v. 0.34 -> first by validator power, descending, then by address, ascending)
    fn sort_validators(vals: &mut [Info]) {
        vals.sort_by_key(|v| (core::cmp::Reverse(v.power), v.address));
    }

    /// Returns the validator with the given Id if its in the Set.
    pub fn validator(&self, val_id: account::Id) -> Option<Info> {
        self.validators
            .iter()
            .find(|val| val.address == val_id)
            .cloned()
    }

    /// Compute the hash of this validator set.
    #[cfg(feature = "rust-crypto")]
    pub fn hash(&self) -> Hash {
        self.hash_with::<crate::crypto::default::Sha256>()
    }

    /// Hash this header with a SHA256 hasher provided by a crypto provider.
    pub fn hash_with<H>(&self) -> Hash
    where
        H: MerkleHash + Sha256 + Default,
    {
        let validator_bytes: Vec<Vec<u8>> = self
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();

        Hash::Sha256(merkle::simple_hash_from_byte_vectors::<H>(&validator_bytes))
    }
}

/// Validator information
// Todo: Remove address and make it into a function that generates it on the fly from pub_key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Info {
    /// Validator account address
    pub address: account::Id,

    /// Validator public key
    pub pub_key: PublicKey,

    /// Validator voting power
    // Compatibility with genesis.json https://github.com/tendermint/tendermint/issues/5549
    #[serde(alias = "voting_power", alias = "total_voting_power")]
    pub power: vote::Power,

    /// Validator name
    pub name: Option<String>,

    /// Validator proposer priority
    #[serde(skip)]
    pub proposer_priority: ProposerPriority,
}

impl Info {
    /// Return the voting power of the validator.
    pub fn power(&self) -> u64 {
        self.power.value()
    }

    /// Verify the given signature against the given sign_bytes using the validators
    /// public key.
    pub fn verify_signature<V>(&self, sign_bytes: &[u8], signature: &Signature) -> Result<(), Error>
    where
        V: Verifier,
    {
        V::verify(self.pub_key, sign_bytes, signature)
            .map_err(|_| Error::signature_invalid("Ed25519 signature verification failed".into()))
    }

    #[cfg(feature = "rust-crypto")]
    /// Create a new validator.
    pub fn new(pk: PublicKey, vp: vote::Power) -> Info {
        Info {
            address: account::Id::from(pk),
            pub_key: pk,
            power: vp,
            name: None,
            proposer_priority: ProposerPriority::default(),
        }
    }
}

/// SimpleValidator is the form of the validator used for computing the Merkle tree.
/// It does not include the address, as that is redundant with the pubkey,
/// nor the proposer priority, as that changes with every block even if the validator set didn't.
/// It contains only the pubkey and the voting power.
/// TODO: currently only works for Ed25519 pubkeys
#[derive(Clone, PartialEq, Eq)]
pub struct SimpleValidator {
    /// Public key
    pub pub_key: PublicKey,
    /// Voting power
    pub voting_power: vote::Power,
}

/// Info -> SimpleValidator
impl From<&Info> for SimpleValidator {
    fn from(info: &Info) -> SimpleValidator {
        SimpleValidator {
            pub_key: info.pub_key,
            voting_power: info.power,
        }
    }
}

impl Info {
    /// Returns the bytes to be hashed into the Merkle tree -
    /// the leaves of the tree.
    pub fn hash_bytes(&self) -> Vec<u8> {
        Protobuf::<RawSimpleValidator>::encode_vec(SimpleValidator::from(self))
    }
}

// Todo: Is there more knowledge/restrictions about proposerPriority?
/// Proposer priority
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct ProposerPriority(i64);

impl From<i64> for ProposerPriority {
    fn from(value: i64) -> Self {
        ProposerPriority(value)
    }
}

impl From<ProposerPriority> for i64 {
    fn from(priority: ProposerPriority) -> i64 {
        priority.value()
    }
}

impl ProposerPriority {
    /// Get the current proposer priority
    pub fn value(self) -> i64 {
        self.0
    }
}

/// A change to the validator set.
///
/// Used to inform Tendermint of changes to the validator set.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#validatorupdate)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Update {
    /// Validator public key
    #[serde(deserialize_with = "deserialize_public_key")]
    pub pub_key: PublicKey,

    /// New voting power
    #[serde(default)]
    pub power: vote::Power,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use pb::{
        abci::ValidatorUpdate as RawValidatorUpdate,
        types::{
            SimpleValidator as RawSimpleValidator, Validator as RawValidator,
            ValidatorSet as RawValidatorSet,
        },
    };
    use super::{Info, Set, SimpleValidator, Update};
    use crate::{prelude::*, Error};

    impl Protobuf<RawValidatorSet> for Set {}

    impl TryFrom<RawValidatorSet> for Set {
        type Error = Error;

        fn try_from(value: RawValidatorSet) -> Result<Self, Self::Error> {
            let validators = value
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?;

            let proposer = value.proposer.map(TryInto::try_into).transpose()?;

            Self::try_from_parts(validators, proposer, value.total_voting_power)
        }
    }

    impl From<Set> for RawValidatorSet {
        fn from(value: Set) -> Self {
            RawValidatorSet {
                validators: value.validators.into_iter().map(Into::into).collect(),
                proposer: value.proposer.map(Into::into),
                total_voting_power: value.total_voting_power.into(),
            }
        }
    }

    impl TryFrom<RawValidator> for Info {
        type Error = Error;

        fn try_from(value: RawValidator) -> Result<Self, Self::Error> {
            Ok(Info {
                address: value.address.try_into()?,
                pub_key: value
                    .pub_key
                    .ok_or_else(Error::missing_public_key)?
                    .try_into()?,
                power: value.voting_power.try_into()?,
                name: None,
                proposer_priority: value.proposer_priority.into(),
            })
        }
    }

    impl From<Info> for RawValidator {
        fn from(value: Info) -> Self {
            RawValidator {
                address: value.address.into(),
                pub_key: Some(value.pub_key.into()),
                voting_power: value.power.into(),
                proposer_priority: value.proposer_priority.into(),
            }
        }
    }

    impl Protobuf<RawSimpleValidator> for SimpleValidator {}

    impl TryFrom<RawSimpleValidator> for SimpleValidator {
        type Error = Error;

        fn try_from(value: RawSimpleValidator) -> Result<Self, Self::Error> {
            Ok(SimpleValidator {
                pub_key: value.pub_key
                    .ok_or_else(Error::missing_public_key)?
                    .try_into()?,
                voting_power: value.voting_power.try_into()?,
            })
        }
    }

    impl From<SimpleValidator> for RawSimpleValidator {
        fn from(value: SimpleValidator) -> Self {
            RawSimpleValidator {
                pub_key: Some(value.pub_key.into()),
                voting_power: value.voting_power.into(),
            }
        }
    }

    impl Protobuf<RawValidatorUpdate> for Update {}

    impl From<Update> for RawValidatorUpdate {
        fn from(vu: Update) -> Self {
            Self {
                pub_key: Some(vu.pub_key.into()),
                power: vu.power.into(),
            }
        }
    }

    impl TryFrom<RawValidatorUpdate> for Update {
        type Error = Error;

        fn try_from(vu: RawValidatorUpdate) -> Result<Self, Self::Error> {
            Ok(Self {
                pub_key: vu
                    .pub_key
                    .ok_or_else(Error::missing_public_key)?
                    .try_into()?,
                power: vu.power.try_into()?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rust-crypto")]
    mod crypto {
        use super::*;

        // make a validator
        fn make_validator(pk: Vec<u8>, vp: u64) -> Info {
            let pk = PublicKey::from_raw_ed25519(&pk).unwrap();
            Info::new(pk, vote::Power::try_from(vp).unwrap())
        }

        #[test]
        fn test_validator_set() {
            // test vector generated by Go code
            // import (
            // "fmt"
            // "github.com/tendermint/tendermint/crypto/ed25519"
            // "github.com/tendermint/tendermint/types"
            // "strings"
            // )
            // func testValSet() {
            // pk1 := ed25519.GenPrivKeyFromSecret([]byte{4, 211, 14, 157, 10, 0, 205, 9, 10, 116, 207,
            // 161, 4, 211, 190, 37, 108, 88, 202, 168, 63, 135, 0, 141, 53, 55, 254, 57, 40, 184, 20,
            // 242}) pk2 := ed25519.GenPrivKeyFromSecret([]byte{99, 231, 126, 151, 159, 236, 2,
            // 229, 33, 44, 200, 248, 147, 176, 13, 127, 105, 76, 49, 83, 25, 101, 44, 57, 20, 215, 166,
            // 188, 134, 94, 56, 165}) pk3 := ed25519.GenPrivKeyFromSecret([]byte{54, 253, 151,
            // 16, 182, 114, 125, 12, 74, 101, 54, 253, 174, 153, 121, 74, 145, 180, 111, 16, 214, 48,
            // 193, 109, 104, 134, 55, 162, 151, 16, 182, 114}) not_in_set :=
            // ed25519.GenPrivKeyFromSecret([]byte{121, 74, 145, 180, 111, 16, 214, 48, 193, 109, 35,
            // 68, 19, 27, 173, 69, 92, 204, 127, 218, 234, 81, 232, 75, 204, 199, 48, 163, 55, 132,
            // 231, 147}) fmt.Println("pk1: ", strings.Join(strings.Split(fmt.Sprintf("%v",
            // pk1.PubKey().Bytes()), " "), ", ")) fmt.Println("pk2:",
            // strings.Join(strings.Split(fmt.Sprintf("%v", pk2.PubKey().Bytes()), " "), ", "))
            // fmt.Println("pk3: ", strings.Join(strings.Split(fmt.Sprintf("%v", pk3.PubKey().Bytes()),
            // " "), ", ")) fmt.Println("not_in_set: ",
            // strings.Join(strings.Split(fmt.Sprintf("%v", not_in_set.PubKey().Bytes()), " "), ", "))
            // v1 := types.NewValidator(pk1.PubKey(), 148151478422287875)
            // v2 := types.NewValidator(pk2.PubKey(), 158095448483785107)
            // v3 := types.NewValidator(pk3.PubKey(), 770561664770006272)
            // set := types.NewValidatorSet([]*types.Validator{v1, v2, v3})
            // fmt.Println("Hash:", strings.Join(strings.Split(fmt.Sprintf("%v", set.Hash()), " "), ",
            // ")) }
            let v1 = make_validator(
                vec![
                    48, 163, 55, 132, 231, 147, 230, 163, 56, 158, 127, 218, 179, 139, 212, 103,
                    218, 89, 122, 126, 229, 88, 84, 48, 32, 0, 185, 174, 63, 72, 203, 52,
                ],
                148_151_478_422_287_875,
            );
            let v2 = make_validator(
                vec![
                    54, 253, 174, 153, 121, 74, 145, 180, 111, 16, 214, 48, 193, 109, 104, 134, 55,
                    162, 151, 16, 182, 114, 125, 135, 32, 195, 236, 248, 64, 112, 74, 101,
                ],
                158_095_448_483_785_107,
            );
            let v3 = make_validator(
                vec![
                    182, 205, 13, 86, 147, 27, 65, 49, 160, 118, 11, 180, 117, 35, 206, 35, 68, 19,
                    27, 173, 69, 92, 204, 224, 200, 51, 249, 81, 105, 128, 112, 244,
                ],
                770_561_664_770_006_272,
            );
            let hash_expect = vec![
                11, 64, 107, 4, 234, 81, 232, 75, 204, 199, 160, 114, 229, 97, 243, 95, 118, 213,
                17, 22, 57, 84, 71, 122, 200, 169, 192, 252, 41, 148, 223, 180,
            ];

            let val_set = Set::without_proposer(vec![v1.clone(), v2.clone(), v3.clone()]);
            let hash = val_set.hash();
            assert_eq!(hash_expect, hash.as_bytes().to_vec());

            let not_in_set = make_validator(
                vec![
                    110, 147, 87, 120, 27, 218, 66, 209, 81, 4, 169, 153, 64, 163, 137, 89, 168,
                    97, 219, 233, 42, 119, 24, 61, 47, 59, 76, 31, 182, 60, 13, 4,
                ],
                10_000_000_000_000_000,
            );

            assert_eq!(val_set.validator(v1.address).unwrap(), v1);
            assert_eq!(val_set.validator(v2.address).unwrap(), v2);
            assert_eq!(val_set.validator(v3.address).unwrap(), v3);
            assert_eq!(val_set.validator(not_in_set.address), None);
            assert_eq!(
                val_set.total_voting_power().value(),
                148_151_478_422_287_875 + 158_095_448_483_785_107 + 770_561_664_770_006_272
            );
        }
    }

    #[test]
    fn deserialize_validator_updates() {
        const FMT1: &str = r#"{
            "pub_key": {
                "Sum": {
                    "type": "tendermint.crypto.PublicKey_Ed25519",
                    "value": {
                        "ed25519": "VqJCr3vjQdffcLIG6RMBl2MgXDFYNY6b3Joaa43gV3o="
                    }
                }
            },
            "power": "573929"
        }"#;
        const FMT2: &str = r#"{
            "pub_key": {
                "type": "tendermint/PubKeyEd25519",
                "value": "VqJCr3vjQdffcLIG6RMBl2MgXDFYNY6b3Joaa43gV3o="
            },
            "power": "573929"
        }"#;

        let update1 = serde_json::from_str::<Update>(FMT1).unwrap();
        let update2 = serde_json::from_str::<Update>(FMT2).unwrap();

        assert_eq!(u64::from(update1.power), 573929);
        assert_eq!(update1, update2);
    }

    #[test]
    fn validator_set_deserialize_all_fields() {
        const VSET: &str = r#"{
            "validators": [
                {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "50",
                    "proposer_priority": "-150"
                },
                {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "42",
                    "proposer_priority": "50"
                }
            ],
            "proposer": {
                "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                },
                "voting_power": "50",
                "proposer_priority": "-150"
            },
            "total_voting_power": "92"
        }"#;

        let vset = serde_json::from_str::<Set>(VSET).unwrap();
        assert_eq!(vset.total_voting_power().value(), 92);
    }

    #[test]
    fn validator_set_deserialize_no_total_voting_power() {
        const VSET: &str = r#"{
            "validators": [
                {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "50",
                    "proposer_priority": "-150"
                },
                {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "42",
                    "proposer_priority": "50"
                }
            ],
            "proposer": {
                "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                },
                "voting_power": "50",
                "proposer_priority": "-150"
            }
        }"#;

        let vset = serde_json::from_str::<Set>(VSET).unwrap();
        assert_eq!(vset.total_voting_power().value(), 92);
    }

    #[test]
    fn validator_set_deserialize_total_voting_power_mismatch() {
        const VSET: &str = r#"{
            "validators": [
                {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "50",
                    "proposer_priority": "-150"
                },
                {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "42",
                    "proposer_priority": "50"
                }
            ],
            "proposer": {
                "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                },
                "voting_power": "50",
                "proposer_priority": "-150"
            },
            "total_voting_power": "100"
        }"#;

        let err = serde_json::from_str::<Set>(VSET).unwrap_err();
        assert!(
            err.to_string()
                .contains("total voting power in validator set does not match the sum of participants' powers"),
            "{err}"
        );
    }

    #[test]
    fn validator_set_deserialize_total_voting_power_exceeds_limit() {
        const VSET: &str = r#"{
            "validators": [
                {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "576460752303423488",
                    "proposer_priority": "-150"
                },
                {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "576460752303423488",
                    "proposer_priority": "50"
                }
            ],
            "proposer": {
                "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                },
                "voting_power": "50",
                "proposer_priority": "-150"
            },
            "total_voting_power": "92"
        }"#;

        let err = serde_json::from_str::<Set>(VSET).unwrap_err();
        assert!(
            err.to_string()
                .contains("total voting power in validator set exceeds the allowed maximum"),
            "{err}"
        );
    }

    #[test]
    fn validator_set_deserialize_total_voting_power_overflow() {
        const VSET: &str = r#"{
            "validators": [
                {
                    "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                    },
                    "voting_power": "6148914691236517205",
                    "proposer_priority": "-150"
                },
                {
                    "address": "026CC7B6F3E62F789DBECEC59766888B5464737D",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "+vlsKpn6ojn+UoTZl+w+fxeqm6xvUfBokTcKfcG3au4="
                    },
                    "voting_power": "6148914691236517205",
                    "proposer_priority": "50"
                },
                {
                    "address": "044EB1BB5D4C1CDB90029648439AEB10431FF295",
                    "pub_key": {
                        "type": "tendermint/PubKeyEd25519",
                        "value": "Wc790fkCDAi7LvZ4UIBAIJSNI+Rp2aU80/8l+idZ/wI="
                    },
                    "voting_power": "6148914691236517206",
                    "proposer_priority": "50"
                }
            ],
            "proposer": {
                "address": "01F527D77D3FFCC4FCFF2DDC2952EEA5414F2A33",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "OAaNq3DX/15fGJP2MI6bujt1GRpvjwrqIevChirJsbc="
                },
                "voting_power": "50",
                "proposer_priority": "-150"
            }
        }"#;

        let err = serde_json::from_str::<Set>(VSET).unwrap_err();
        assert!(
            err.to_string()
                .contains("total voting power in validator set exceeds the allowed maximum"),
            "{err}"
        );
    }
}
