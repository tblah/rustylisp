//! Root crate for the library

#![feature(trace_macros)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy::pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::linkedlist))]
#![crate_name = "rustyscheme"]
#![crate_type = "lib"]
#![warn(missing_docs)]
#![warn(non_upper_case_globals)]
#![warn(non_camel_case_types)]
#![warn(unused_qualifications)]

pub mod ast;
pub mod data;
pub mod stdlib;
pub mod tokenise;

/// Possible parse errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The token stream was empty (not fatal)
    EmptyStream,
    /// Expected token but the stream was empty (fatal)
    MissingToken,
    /// Encountered the end of the token iterator before we thought we were done
    PartialStream,
    /// Found an unexpected ')'
    ClosingBracket,
    /// Syntax Error e.g. #a
    SyntaxError(String),
}

/// Creates a `ParseError::SyntaxError`
impl<'a> From<&'a str> for ParseError {
    fn from(s: &'a str) -> Self {
        ParseError::SyntaxError(String::from(s))
    }
}

/// Creates a `ParseError::SyntaxError`
impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::SyntaxError(s)
    }
}
