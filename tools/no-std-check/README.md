# `no_std` Compliance Check

This crate checks the `no_std` compliance of the supported crates in ibc-rs.

## Make Recipes

- `check-panic-conflict` - Check for `no_std` compliance by installing a panic handler, and any other crate importing `std` will cause a conflict. Runs on default target.

- `check-cargo-build-std` - Check for `no_std` compliance using Cargo nightly's `build-std` feature. Runs on the target `x86_64-unknown-linux-gnu`.

- `check-wasm` - Check for WebAssembly and `no_std` compliance by building on the target `wasm32-unknown-unknown` and installing a panic handler.

- `check-substrate` - Check for Substrate, WebAssembly, and `no_std` compliance by importing Substrate crates and building on `wasm32-unknown-unknown`. Any crate using `std` will cause a conflict on the panic and out-of-memory (OOM) handlers installed by `sp-io`.

## Conflict Detection Methods

There are two methods that we use to detect `std` conflict:

### Panic Handler Conflict

This follows the outline of the guide by
[Danilo Bargen](https://blog.dbrgn.ch/2019/12/24/testing-for-no-std-compatibility/)
to register a panic handler in the `no-std-check` crate.
Any crate imported `no-std-check` that uses `std` will cause a compile error that
looks like follows:

```
$ cargo build
    Updating crates.io index
   Compiling no-std-check v0.1.0 (/data/development/informal/ibc-rs/no-std-check)
error[E0152]: found duplicate lang item `panic_impl`
  --> src/lib.rs:31:1
   |
31 | fn panic(_info: &PanicInfo) -> ! {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: the lang item is first defined in crate `std` (which `prost` depends on)
   = note: first definition in `std` loaded from /home/ubuntu/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd-b6b48477bfa8c673.rlib
   = note: second definition in the local crate (`no_std_check`)

error: aborting due to previous error

For more information about this error, try `rustc --explain E0152`.
error: could not compile `no-std-check`
```

- Pros:
  - Can be tested using Rust stable.
- Cons:
  - Crates must be listed on both `Cargo.toml` and `lib.rs`.
  - Crates that are listed in `Cargo.toml` but not imported inside `lib.rs` are not checked.

### Override std crates using Cargo Nightly

This uses the unstable `build-std` feature provided by
[Cargo Nightly](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std).
With this we can explicitly pass the std crates we want to support, `core` and `alloc`,
via command line, and exclude the `std` crate.

If any of the dependency uses `std`, they will fail to compile at all, albeit with
confusing error messages:

```
$ rustup run nightly -- cargo build -j1 -Z build-std=core,alloc --target x86_64-unknown-linux-gnu
   ...
   Compiling bytes v1.0.1
error[E0773]: attempted to define built-in macro more than once
    --> /home/ubuntu/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/macros/mod.rs:1201:5
     |
1201 | /     macro_rules! cfg {
1202 | |         ($($cfg:tt)*) => {
1203 | |             /* compiler built-in */
1204 | |         };
1205 | |     }
     | |_____^
     |
note: previously defined here
    --> /home/ubuntu/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/macros/mod.rs:1201:5
     |
1201 | /     macro_rules! cfg {
1202 | |         ($($cfg:tt)*) => {
1203 | |             /* compiler built-in */
1204 | |         };
1205 | |     }
     | |_____^

error: duplicate lang item in crate `core` (which `std` depends on): `bool`.
  |
  = note: the lang item is first defined in crate `core` (which `bytes` depends on)
  = note: first definition in `core` loaded from /data/development/informal/ibc-rs/no-std-check/target/x86_64-unknown-linux-gnu/debug/deps/libcore-c00d94870d25cd7e.rmeta
  = note: second definition in `core` loaded from /home/ubuntu/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-9924c22ae1efcf66.rlib

error: duplicate lang item in crate `core` (which `std` depends on): `char`.
  |
  = note: the lang item is first defined in crate `core` (which `bytes` depends on)
  = note: first definition in `core` loaded from /data/development/informal/ibc-rs/no-std-check/target/x86_64-unknown-linux-gnu/debug/deps/libcore-c00d94870d25cd7e.rmeta
  = note: second definition in `core` loaded from /home/ubuntu/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-9924c22ae1efcf66.rlib
...
```

The above error are shown when building the `bytes` crate. This is caused by `bytes` using imports from `std`,
which causes `std` to be included and produce conflicts with the `core` crate that is explicitly built by Cargo.
This produces very long error messages, so we may want to use tools like `less` to scroll through the errors.

Pros:
  - Directly identify use of `std` in dependencies.
  - Error is raised on the first dependency that imports `std`.

Cons:
  - Nightly-only feature that is subject to change.
  - Confusing and long error messages.
