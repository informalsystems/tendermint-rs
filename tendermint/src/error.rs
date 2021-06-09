//! Error types

use crate::account;
use crate::vote;
use flex_error::*;
use std::string::String;
pub type Error = anyhow::Error;

define_error! {
    #[derive(Debug)]
    KindError {
        Crypto
        [DisplayError<Error>]
        |_| { format_args!("cryptographic error") },

        InvalidKey
        [DisplayError<Error>]
        |_| { format_args!("invalid key") },

        Io
        [DisplayError<Error>]
        |_| { format_args!("I/O error") },

        Length
        [DisplayError<Error>]
        |_| { format_args!("length error") },

        Parse
        {data: String}
        | e | { format_args!("error parsing data {}", e.data) },

        ParseInt
        {data: String}
        [DisplayError<std::num::ParseIntError>]
        | e | { format_args!("error parsing int data: {}", e.data) },

        ParseUrl
        [DisplayError<url::ParseError>]
        |_| { format_args!("error parsing url error") },

        Protocol
        [DisplayError<Error>]
        |_| { format_args!("protocol error") },

        OutOfRange
        [DisplayError<Error>]
        |_| { format_args!("value out of range") },

        SignatureInvalid
        [DisplayError<Error>]
        |_| { format_args!("bad signature") },

        InvalidMessageType
        [DisplayError<Error>]
        |_| { format_args!("invalid message type") },

        NegativeHeight
        [DisplayError<Error>]
        |_| { format_args!("negative height") },

        NegativeRound
        [DisplayError<Error>]
        |_| { format_args!("negative round") },

        NegativePolRound
        [DisplayError<Error>]
        |_| { format_args!("negative POL round") },

        NegativeValidatorIndex
        |_| { format_args!("negative validator index") },

        InvalidHashSize
        [DisplayError<Error>]
        |_| { format_args!("invalid hash: expected hash size to be 32 bytes") },

        NoTimestamp
        |_| { format_args!("no timestamp") },

        InvalidTimestamp
        [DisplayError<Error>]
        |_| { format_args!("invalid timestamp") },

        InvalidAccountIdLength
        [DisplayError<Error>]
        |_| { format_args!("invalid account ID length") },

        InvalidSignatureIdLength
        [DisplayError<Error>]
        |_| { format_args!("invalid signature ID length") },

        IntegerOverflow
        |_| { format_args!("integer overflow") },

        NoVoteFound
        [DisplayError<Error>]
        |_| { format_args!("no vote found") },

        NoProposalFound
        [DisplayError<Error>]
        |_| { format_args!("no proposal found") },

        InvalidAppHashLength
        [DisplayError<Error>]
        |_| { format_args!("invalid app hash length") },

        InvalidPartSetHeader
        [DisplayError<Error>]
        |_| { format_args!("invalid part set header") },

        MissingHeader
        [DisplayError<Error>]
        |_| { format_args!("missing header field") },

        MissingData
        [DisplayError<Error>]
        |_| { format_args!("missing data field") },

        MissingEvidence
        [DisplayError<Error>]
        |_| { format_args!("missing evidence field") },

        MissingTimestamp
        [DisplayError<Error>]
        |_| { format_args!("missing timestamp field") },

        InvalidBlock
        [DisplayError<Error>]
        |_| { format_args!("invalid block") },

        InvalidFristBlock
        [DisplayError<Error>]
        |_| { format_args!("invalid first block") },

        MissingVersion
        [DisplayError<Error>]
        |_| { format_args!("missing version") },

        InvalidHeader
        [DisplayError<Error>]
        |_| { format_args!("invalid header") },

        InvalidFirstHeader
        [DisplayError<Error>]
        |_| { format_args!("invalid first header") },

        InvalidSignature
        [DisplayError<Error>]
        |_| { format_args!("invalid signature") },

        InvalidValidatorAddress
        [DisplayError<Error>]
        |_| { format_args!("invalid validator address") },

        InvalidSignedHeader
        [DisplayError<Error>]
        |_| { format_args!("invalid signed header") },

        InvalidEvidence
        [DisplayError<Error>]
        |_| { format_args!("invalid evidence") },

        BlockIdFlag
        [DisplayError<Error>]
        |_| { format_args!("invalid block id flag") },

        NegativePower
        [DisplayError<Error>]
        |_| { format_args!("negative power") },

        RawVotingPowerMismatch
        { raw: vote::Power, computed: vote::Power}
        [DisplayError<Error>]
        |e| { format_args!("mismatch between raw voting ({0:?}) and computed one ({1:?})", e.raw, e.computed) },

        MissingPublicKey
        [DisplayError<Error>]
        |_| { format_args!("missing public key") },

        InvalidValidatorParams
        [DisplayError<Error>]
        |_| { format_args!("invalid validator parameters") },

        InvalidVersionParams
        [DisplayError<Error>]
        |_| { format_args!("invalid version parameters") },

        NegativeMaxAgeNum
        [DisplayError<Error>]
        |_| { format_args!("negative max_age_num_blocks") },

        MissingMaxAgeDuration
        [DisplayError<Error>]
        |_| { format_args!("missing max_age_duration") },

        ProposerNotFound
        {account: account::Id}
        [DisplayError<Error>]
        |e| { format_args!("proposer with address '{0}' no found in validator set", e.account) },

        InFallible
        [DisplayError<Error>]
        |_| { format_args!("infallible") },

        ChronoParse
        [DisplayError<Error>]
        |_| { format_args!("chrono parse error") },

        SubtleEncoding
        [DisplayError<Error>]
        |_| { format_args!("subtle encoding error") },

        SerdeJson
        [DisplayError<Error>]
        |_| { format_args!("serde json error") },

        Toml
        [DisplayError<Error>]
        |_| { format_args!("toml de error") },
    }
}
