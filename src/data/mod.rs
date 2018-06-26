//! Data model

use std::collections::LinkedList;

pub mod env;

/// Representation of a scheme object
#[derive(Debug, PartialEq, Clone)]
pub enum SchemeObject {
    /// A boolean value
    Bool(bool),
    /// A symbol e.g. 'HELLO
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
