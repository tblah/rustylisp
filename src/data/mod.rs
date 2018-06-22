//! Data model
use std::collections::LinkedList;

/// Representation of a scheme object
#[derive(Debug, PartialEq)]
pub enum SchemeObject {
    /// A boolean value
    Bool(bool),
    /// A symbol e.g. 'HELLO
    Symbol(String),
    /// A string e.g. "HELLO"
    String(String),
    /// A string form e.g. (add 1 2)
    Form(SchemeForm),
}

/// A form: i.e. something that lives in brackets (). This could be code, a list, or a vector
#[derive(Debug, PartialEq)]
pub enum SchemeForm {
    /// A linked list ()
    List(SchemeList),
    /// A vector #()
    Vector(Vec<SchemeObject>),
}

/// A linked list: this could be code (add 1 2) or a quoted list '(1 2 3)
#[derive(Debug, PartialEq)]
pub struct SchemeList {
    quoted: bool,
    list: LinkedList<SchemeObject>,
}
