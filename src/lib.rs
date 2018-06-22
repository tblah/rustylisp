//! Root crate for the library

#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(linkedlist))]
#![crate_name = "rustyscheme"]
#![crate_type = "lib"]
#![warn(missing_docs)]
#![warn(non_upper_case_globals)]
#![warn(non_camel_case_types)]
#![warn(unused_qualifications)]
pub mod ast;
pub mod data;
pub mod tokenise;
