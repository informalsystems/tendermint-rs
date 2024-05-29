- `[tendermint-light-client-verifier]` Restores the commit verification interfaces of `PredicateVerifier<P, C, V>` from `<= 0.35.0`.
  * `verify_commit(&self. untrusted: &UntrustedBlockState<'_>)` is restored, as in <= 0.35.0.
  * `verify_commit(&self, untrusted: &UntrustedBlockState<'_>, trusted: &TrustedBlockState<'_>,)` introduced in 0.36.0 is renamed to `verify_commit_against_trusted`.
  The performance improvements made in the `0.36.0` release are still intact.
  ([\#1423](https://github.com/informalsystems/tendermint-rs/pull/1423))
