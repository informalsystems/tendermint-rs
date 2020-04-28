#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(
     unreachable_pub,
     // missing_docs,
     // missing_doc_code_examples
 )]
#![allow(clippy::too_many_arguments, clippy::match_wild_err_arm)]

pub mod components;
pub mod errors;
pub mod event;
pub mod operations;
pub mod predicates;
pub mod prelude;
pub mod trace;
pub mod trusted_store;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
