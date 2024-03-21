# ADR 010: Improvements of Time API, internal representation, and serialization

## Changelog

* 2021-11-29: Created the ADR.

## Context

The `Time` type is defined in `tendermint` to provide a data type for time
calculations with better safety and ergonomics than the `prost`-dictated
`Timestamp` struct, defined in `tendermint-proto` based on Google's
common protobuf message description.
[Concerns](https://github.com/informalsystems/tendermint-rs/issues/865) have
been raised about the need for such a type in the API; however, as we
currently lack a well-supported library in the Rust ecosystem that would provide
the desirable formatting through serde and enforce a range of usable values,
the domain type seems to be necessary.

The current API of `Time` has the following problems:

* It's a newtype struct publicly exposing a `DateTime<Utc>` inner value
  provided by the `chrono` crate. `chrono` has a known
  [issue][RUSTSEC-2020-0159] with soundness and security and there is no ETA yet
  on getting it fixed.
  The dependency on chrono triggers cargo audit failures for every project
  using the `tendermint` library.

* The range of usable values supported by `Time` has not been explicitly
  defined or enforced. Currently `Time` allows values which can't have
  a valid RFC 3339 representation and whose equivalent Unix timestamp values
  are not allowed by the Google protobuf specification of `Timestamp`.

* The serde implementations for `Time` and the proto `Timestamp` struct
  always serialize or expect the value as a string, even in serialization
  formats that are not human-readable and allow a more efficient binary
  representation.

* Arithmetic operators have been provided for `Time` via `Add`/`Sub` trait
  implementations. However, the `Result` output type is
  [surprising][guideline-overload] and makes for poor usability of overloaded
  operators.

* Arithmetic and comparison operations are much slower on parsed date/time
  structures than on an integer timestamp representation.
  If `Time` is meant to be used in performance-sensitive workloads
  for operations with date-time values, such as offsetting by a duration,
  time difference, or time comparisons, its internal representation should be
  more optimized for integer arithmetic.

* The `Time::as_rfc3339` conversion method is
  [named improperly][guideline-naming] with regard to Rust naming guidelines.

[RUSTSEC-2020-0159]: https://rustsec.org/advisories/RUSTSEC-2020-0159.html
[guideline-overload]: https://rust-lang.github.io/api-guidelines/predictability.html#c-overload
[guideline-naming]: https://rust-lang.github.io/api-guidelines/naming.html#c-conv

## Decision

Make these changes, possibly in steps:

* Make the inner member(s) of the `Time` struct private.

* Specify that only date-times in the year range 1-9999 inclusive
  can be represented by a `Time` value. This matches the restrictions
  specified in the Google protobuf message definition for `Timestamp`.

* Remove conversions from/to `chrono::DateTime`,
  introducing these impls instead:
  * `impl TryFrom<time::OffsetDateTime> for Time`
    (fallible due to the additional range restrictions)
  * `impl From<Time> for time::OffsetDateTime`

* Change the serde implementations for `tendermint_proto::Timestamp`
  (and, by deriving-via-conversions, for `tendermint::Time`) to use
  member-wise struct serialization on non-human-readable serializers.

* Remove the `Add` and `Sub` impls for `Time`, replacing the operator
  overloading with `checked_add` and `checked_sub` methods in the fashion of
  `SystemTime` and `time::OffsetDateTime`. This provides a speed bump for the
  API users who must take care of potential range overflows, in a way that
  does not force parentheses appended with error handling cruft nested
  through the expression tree.

* Change the internal representation of `Time` to an `i128` Unix timestamp
  in nanoseconds.

* Rename `Time::as_rfc3339` back to `Time::to_rfc3339`.
  Also in `tendermint_proto`, the public helper function named
  `serializers::timestamp::as_rfc3339_nanos` should be similarly renamed.

## Status

Proposed

## Consequences

### Positive

* `Time` gets a clear purpose and a documented validity range.
* The implementation details of `Time` are made private and can be changed
  in the future with little or no breakage for the API consumers.
* Interfacing with ecosystem libraries for time computations can be made
  optional, enabled via feature-gated conversions. In particular, `chrono`
  can be completely cut out until its `localtime_r` issue is fixed,
  giving the Tendermint library consumers their `cargo audit` peace.
* The `checked_add` and `checked_sub` methods replacing arithmetic operators
  over `Time` are conventional and chainable.
* Duration arithmetic over integer timestamp values is fast.
* Changing the internal representation to a Unix timestamp puts treatment of
  leap seconds outside of tendermint-rs, making it aligned with representation
  of time in the Tendermint protocol. None of Rust libraries explicitly
  support Google's [24-hour linear smear][google-smear] over standard UTC.
  `chrono` has [its own ideas][chrono-leap] about representing leap seconds,
  which are rather complex and hard to use correctly.

[google-smear]: https://developers.google.com/time/smear
[chrono-leap]: https://docs.rs/chrono/0.4.19/chrono/naive/struct.NaiveTime.html#leap-second-handling

### Negative

* Breaking changes in the API.
* Application developers who prefer `chrono` despite its faults lose
  convenient ways to convert between `Time` and `chrono::DateTime`.
* The timestamp-based internal representation of `Time` is less
  memory-efficient than a parsed time struct.

### Inconclusive

* Performance of code that needs to retrieve a human time representation
  out of `Time` (or `Timestamp`) will be affected. The only such use cases
  supported internally by the redesigned `Time` API are parsing and formatting,
  including the serde implementations for human-readable formats;
  in these cases, the added conversion overhead should be negligible
  in proportion to other parsing/formatting logic.
  For other uses, the application can convert a `Time` value to
  `time::OffsetDateTime` (or, potentially, `chrono::DateTime`, or any other
  foreign crate types that may become popular in the future) and use that
  value for further computations.

## References

### Issues

* https://github.com/informalsystems/tendermint-rs/issues/865
* https://github.com/informalsystems/tendermint-rs/issues/1008
* https://github.com/informalsystems/tendermint-rs/issues/1012

### Implementation PRs

* https://github.com/informalsystems/tendermint-rs/pull/1030
