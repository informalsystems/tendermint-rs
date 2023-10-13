- Don’t enable `flex-error/eyre_tracer` feature in crates which don’t
  use eyre directly.  If you’re using eyre, and no other crate enables
  it, you may need to enable that explicitly.
  ([\#1371](https://github.com/informalsystems/tendermint-rs/pull/1371))
