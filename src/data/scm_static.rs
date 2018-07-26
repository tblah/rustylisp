//! Defines `SchemeObject`

use std::collections::LinkedList;
use std::fmt;

/// Representation of a scheme object as parsed from source code
#[derive(PartialEq, Clone)]
pub enum SchemeObject {
    /// A boolean value
    Bool(bool),
    /// A symbol e.g. a variable name
    Symbol(String),
    /// A string e.g. "HELLO"
    String(String),
    /// A linked list which is not quoted: (...)
    List(LinkedList<SchemeObject>),
    /// A quoted object
    Quoted(Box<SchemeObject>),
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

impl fmt::Display for SchemeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SchemeObject::*;
        match *self {
            Bool(b) => if b {
                write!(f, "#t")
            } else {
                write!(f, "#f")
            },
            Symbol(ref s) | String(ref s) => write!(f, "{}", s),
            List(ref lst) => {
                super::print_code_lst(f, lst.iter().map(|x| format!("{:?}", x)), ['(', ')'])
            }
            Quoted(ref scm_obj) => {
                write!(f, "'{}", scm_obj)
            }
            Vector(ref lst) => {
                super::print_code_lst(f, lst.iter().map(|x| format!("{:?}", x)), ['[', ']'])
            }
        }
    }
}

impl fmt::Debug for SchemeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SchemeObject::*;
        match *self {
            String(ref s) => write!(f, "\"{}\"", s),
            _ => fmt::Display::fmt(&self, f),
        }
    }
}
