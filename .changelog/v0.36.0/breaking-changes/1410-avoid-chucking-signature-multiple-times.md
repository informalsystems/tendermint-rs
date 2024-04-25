- `[light-client-verifier]` Rework VerificationPredicates and VotingPowerCalculator
  by introducing methods which check validators and signers overlap at once.
  The motivation of this is to avoid checking the same signature multiple
  times.

  Consider a validator is in old and new set.  Previously their signature would
  be verified twice.  Once by call to `has_sufficient_validators_overlap`
  method and second time by call to `has_sufficient_signers_overlap` method.

  With the new interface, `has_sufficient_validators_and_signers_overlap` is
  called and it can be implemented to remember which signatures have been
  verified.

  As a side effect of those changes, signatures are now verified in the order
  of validator’s power which may further reduce number of signatures which
  need to be verified.

  ([\#1410](https://github.com/informalsystems/tendermint-rs/pull/1410))
