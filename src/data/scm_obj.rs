//! Defines `SchemeObject` type and implements basic traits

use data::env::*;

use std::cmp::PartialEq;
use std::collections::LinkedList;
use std::fmt;
use std::rc::Rc;

/// Representation of a scheme object
#[derive(Clone)]
pub enum SchemeObject {
    /// A boolean value
    Bool(bool),
    /// A symbol e.g. a variable name
    Symbol(String),
    /// A string e.g. "HELLO"
    String(String),
    /// A linked list
    List(LinkedList<SchemeObject>),
    /// A quoted object
    Quoted(Box<SchemeObject>),
    /// A vector #()
    Vector(Vec<SchemeObject>),
    /// A built-in (rust) function
    RFunc(
        String,                                                            // Name
        fn(&LinkedList<Rc<SchemeObject>>, &PackedEnv) -> Rc<SchemeObject>, // Function pointer
    ),
    /// A scheme function
    SFunc(
        Box<SchemeObject>, // Code list
        Vec<String>,       // argument names
        // closure environment (the environment in use when the function was defined
        PackedEnv,
    ),
    /// None (for use as a function return value)
    None,
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

/// We can't #[derive(PartialEq)] because `PartialEq` for `fn` isn't working
/// TODO check if this is fixed and if not, file a bug report
impl PartialEq for SchemeObject {
    fn eq(&self, other: &Self) -> bool {
        use self::SchemeObject::*;

        match (self, other) {
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Symbol(s1), Symbol(s2)) | (String(s1), String(s2)) => s1 == s2,
            (List(l1), List(l2)) => l1 == l2,
            (Quoted(o1), Quoted(o2)) => o1 == o2,
            (Vector(v1), Vector(v2)) => v1 == v2,
            (RFunc(_, f1), RFunc(_, f2)) => *f1 as usize == *f2 as usize, // lifted from rust stdlib
            (SFunc(b1, v1, e1), SFunc(b2, v2, e2)) => b1 == b2 && v1 == v2 && e1 == e2,
            (None, None) => true,
            _ => false,
        }
    }
}

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

/// For normal printing
impl fmt::Display for SchemeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SchemeObject::*;

        match self {
            Bool(b) => if *b {
                write!(f, "#t")
            } else {
                write!(f, "#f")
            },
            Symbol(ref s) | String(ref s) => write!(f, "{}", s),
            List(ref lst) => print_code_lst(f, lst.iter().map(|x| format!("{:?}", x)), ['(', ')']),
            Quoted(ref scm_obj) => write!(f, "'{}", scm_obj),
            Vector(ref lst) => {
                print_code_lst(f, lst.iter().map(|x| format!("{:?}", x)), ['[', ']'])
            }
            RFunc(_, _) => write!(f, "Built-in function: {:?}", &self),
            SFunc(obj, names, _) => {
                // "(lambda ({}) {})"
                write!(f, "(lambda ")?;
                print_code_lst(f, names.iter().cloned(), ['(', ')'])?;
                write!(f, " {}", obj)
            }
            None => Ok(()),
        }
    }
}

/// For printing source code (e.g. displaying a scheme function as a lambda expression)
impl fmt::Debug for SchemeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SchemeObject::*;
        match self {
            String(s) => write!(f, "\"{}\"", s),
            RFunc(name, _) => write!(f, "{}", name),
            _ => fmt::Display::fmt(&self, f),
        }
    }
}
