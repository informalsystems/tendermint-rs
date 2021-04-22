# anomaly.rs 🦠 <a href="https://www.iqlusion.io"><img src="https://storage.googleapis.com/iqlusion-production-web/img/logo/iqlusion-rings-sm.png" alt="iqlusion" width="24" height="24"></a>

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][rustc-image]
[![Safety Dance][safety-image]][safety-link]
[![Build Status][build-image]][build-link]

Error context library with support for type-erased sources and backtraces,
targeting full support of all features on stable Rust, and with an eye towards
serializing runtime errors using `serde`.

[Documentation][docs-link]

## About

**anomaly.rs** draws inspiration from libraries like [`error-chain`],
[`failure`], and [`anyhow`] to provide the following features:

- An [`anomaly::Context`] type which impls [`std::error::Error`] including
  support for type-erased [`anomaly::BoxError`] sources. Contexts are generic
  around an error `Kind`, making the sources optional, and generally trying
  to strike a balance between typed errors and `Box`-based type erasure.
- Stringly typed errors using the [`anomaly::Message`] type, with a set
  of macros to construct these errors.
- Backtrace support using the [`backtrace`] crate, and with it support for
  stable Rust where other libraries might require nightly.
- Support for serializing errors using `serde`, allowing them to be submitted
  to exception reporting services and other structured logging systems.

Notably **anomaly.rs** does NOT include any sort of proc macro to define
its error `Kind` type. We recommend [`thiserror`] for that purpose.

## What makes anomaly.rs different?

[`anomaly::Context`] and its `Box`-ed wrapper, [`anomaly::Error`], are
generic around a concrete `Kind` type. Type erasure (based on
[`std::error::Error`]) is only used when constructing error chains:

- Concrete (generic) types for immediate errors
- Type erasure for error sources
- No additional traits beyond `std::error::Error`
- Stringly typed [`anomaly::Message`] for where enum variants are too
  cumbersome or error messages are coming from e.g. API responses.
- Structured logging of your errors using `serde`

## History

**anomaly.rs** is an extraction of a set of patterns and boilerplate
from real-world libraries and applications, most notably [Abscissa].

## Minimum Supported Rust Version

Rust **1.47** or newer.

In the future, we reserve the right to change MSRV (i.e. MSRV is out-of-scope
for this crate's SemVer guarantees), however when we do it will be accompanied by
a minor version bump.

## License

Copyright © 2019-2021 iqlusion

**anomaly.rs** is distributed under the terms of either the MIT license
or the Apache License (Version 2.0), at your option.

See [LICENSE] (Apache License, Version 2.0) file in the `iqlusioninc/crates`
toplevel directory of this repository or [LICENSE-MIT] for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you shall be dual licensed as above,
without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/anomaly.svg
[crate-link]: https://crates.io/crates/anomaly
[docs-image]: https://docs.rs/anomaly/badge.svg
[docs-link]: https://docs.rs/anomaly/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.47+-blue.svg
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[build-image]: https://github.com/iqlusioninc/crates/actions/workflows/anomaly.yml/badge.svg
[build-link]: https://github.com/iqlusioninc/crates/actions/workflows/anomaly.yml

[//]: # (general links)

[`error-chain`]: https://crates.io/crates/error-chain
[`failure`]: https://crates.io/crates/failure
[`anyhow`]: https://crates.io/crates/anyhow
[`anomaly::Context`]: https://docs.rs/anomaly/latest/anomaly/struct.Context.html
[`anomaly::Error`]: https://docs.rs/anomaly/latest/anomaly/struct.Error.html
[`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
[`anomaly::BoxError`]: https://docs.rs/anomaly/latest/anomaly/type.BoxError.html
[`anomaly::Message`]: https://docs.rs/anomaly/latest/anomaly/struct.Message.html
[`backtrace`]: https://crates.io/crates/backtrace
[`thiserror`]: https://crates.io/crates/thiserror
[Abscissa]: https://github.com/iqlusioninc/abscissa
[LICENSE]: https://github.com/iqlusioninc/crates/blob/main/LICENSE
[LICENSE-MIT]: https://github.com/iqlusioninc/crates/blob/main/anomaly/LICENSE-MIT
