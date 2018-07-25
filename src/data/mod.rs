//! Data model

pub mod env;
pub mod exec;
pub mod runtime;
pub mod scm_static;

use std::fmt;

/// Utility fn for impls of `fmt::{Debug, Display}`
/// Prints an iterator of strings producing something like (one two three) or [one two]
fn print_code_lst<I>(f: &mut fmt::Formatter, mut lst: I, s: [char; 2]) -> fmt::Result
where
    I: Iterator<Item = String>,
{
    // opening symbol
    write!(f, "{}", s[0])?;

    // no space on first iter
    if let Some(so) = lst.next() {
        write!(f, "{}", so)?;
    }
    // spaces on subsequent iters
    for so in lst {
        write!(f, " {}", so)?;
    }

    // closing symbol
    write!(f, "{}", s[1])
}
