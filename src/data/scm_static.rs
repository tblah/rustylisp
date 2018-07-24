//! Defines `SchemeObject`

use std::collections::LinkedList;
use std::fmt;

/// Representation of a scheme object as parsed from source code
#[derive(Debug, PartialEq, Clone)]
pub enum SchemeObject {
    /// A boolean value
    Bool(bool),
    /// A symbol e.g. a variable name
    Symbol(String),
    /// A string e.g. "HELLO"
    String(String),
    /// A linked list which is not quoted: (...)
    CodeList(LinkedList<SchemeObject>),
    /// A linked list which is quoted: '(...)
    QuotedList(LinkedList<SchemeObject>),
    /// A vector #()
    Vector(Vec<SchemeObject>),
}

/// Creates a `SchemeObject::String`
impl<'a> From<&'a str> for SchemeObject {
    fn from(s: &str) -> Self {
        SchemeObject::String(String::from(s))
    }
}

/// Creates a `SchemeObject::String`
impl<'a> From<&'a String> for SchemeObject {
    fn from(s: &String) -> Self {
        SchemeObject::String(s.clone())
    }
}

/// Creates a `SchemeObject::String`
impl From<String> for SchemeObject {
    fn from(s: String) -> Self {
        SchemeObject::String(s)
    }
}

/// Creates a `SchemeObject::Bool`
impl From<bool> for SchemeObject {
    fn from(b: bool) -> Self {
        SchemeObject::Bool(b)
    }
}

/// Types that can be turned into a `SchemeObject::Symbol` without error
pub trait SymFrom<T> {
    /// `Create a SchemeObject::Symbol`
    fn sym_from(s: T) -> SchemeObject;
}

impl<'a> SymFrom<&'a str> for SchemeObject {
    fn sym_from(s: &str) -> Self {
        SchemeObject::Symbol(String::from(s))
    }
}

impl<'a> SymFrom<&'a String> for SchemeObject {
    fn sym_from(s: &String) -> Self {
        SchemeObject::Symbol(s.clone())
    }
}

impl SymFrom<String> for SchemeObject {
    fn sym_from(s: String) -> Self {
        SchemeObject::Symbol(s)
    }
}

/// Utility fn for impl of `fmt::Display` for `SchemeObject`
fn print_code_lst<'a, I>(f: &mut fmt::Formatter, mut lst: I, s: [char; 2]) -> fmt::Result
where
    I: Iterator<Item = &'a SchemeObject>,
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

impl fmt::Display for SchemeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SchemeObject::*;
        match *self {
            Bool(b) => if b {
                write!(f, "#t")
            } else {
                write!(f, "#f")
            },
            Symbol(ref s) => write!(f, "{}", s),
            String(ref s) => write!(f, "\"{}\"", s),
            CodeList(ref lst) => print_code_lst(f, lst.iter(), ['(', ')']),
            QuotedList(ref lst) => {
                write!(f, "'")?;
                print_code_lst(f, lst.iter(), ['(', ')'])
            }
            Vector(ref lst) => print_code_lst(f, lst.iter(), ['[', ']']),
        }
    }
}
