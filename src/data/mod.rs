//! Data model
use std::collections::LinkedList;

/// Representation of a scheme object
pub enum SchemeObject {
    Bool(bool),
    Symbol(String),
    String(String),
    Form(SchemeForm),
}

pub enum SchemeForm {
    List(SchemeList),
    Vector(Vec<SchemeObject>)
}

pub struct SchemeList {
    quoted: bool,
    list: LinkedList<SchemeObject>,
}

