- [`tendermint-light-client-verifier`] Add `Verifier::verify_misbehaviour_header`
  for verifying headers coming from a misbehaviour evidence.
  The verification for these headers is a bit more relaxed in order to catch FLA attacks.
  In particular the "header in the future" check for the header should be skipped.
  ([\#1294](https://github.com/informalsystems/tendermint-rs/issues/1294))
