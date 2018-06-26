//! Data model

pub mod env;
pub mod runtime;

use self::env::*;
use self::runtime::RuntimeObject;
use super::ParseError;
use std::collections::LinkedList;
use std::rc::Rc;

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

impl SchemeObject {
    /// If it is a symbol, look it up and return the result
    /// If it is a CodeList, execute it and return the result
    /// Otherwise return as-is
    pub fn exec(&self, env: &mut Environment) -> Result<Rc<RuntimeObject>, ParseError> {
        match self {
            // look up the symbol name in the environment
            SchemeObject::Symbol(s) => env
                .lookup(&s)
                .ok_or_else(|| ParseError::NameLookup(s.clone())),
            SchemeObject::CodeList(lst) => exec_codelist(&lst, env),
            x => Ok(Rc::new(RuntimeObject::SchemeObject(x.clone()))), // TODO remove clone by listing Rc<ScmObj>?
        }
    }
}

/// helper function for `SchemeObject::exec`
fn exec_codelist(
    lst: &LinkedList<SchemeObject>,
    env: &mut Environment,
) -> Result<Rc<RuntimeObject>, ParseError> {
    let scm_obj = match lst.front() {
        Some(c) => c,
        None => {
            return Err(ParseError::SyntaxError(String::from(
                "Executing empty codelist",
            )))
        }
    };

    if let SchemeObject::Symbol(_cmd) = scm_obj {
        // TODO special forms if, let, etc

        // look up the function
        let mut runtime_obj = scm_obj.exec(env)?;

        // get list tail - TODO immutable single linked list to avoid clone
        let tail = if lst.len() > 1 {
            lst.clone().split_off(1)
        } else {
            LinkedList::new()
        };

        // execute the function
        runtime_obj.exec(&tail, env)
    } else {
        Err(ParseError::SyntaxError(format!(
            "{:?} found; function name expected",
            scm_obj
        )))
    }
}

#[cfg(test)]
mod test {
    use data::env::Environment;
    use data::runtime::RuntimeObject;
    use data::*;

    #[test]
    fn resolve_symbol() {
        let name = String::from("name");
        let val = String::from("value");

        let symbol = SchemeObject::Symbol(name.clone());
        let val_obj = SchemeObject::String(val.clone());

        // create an environment where "name" is mapped to "value"
        let mut env = Environment::new(None);
        let get_entry = || RuntimeObject::SchemeObject(val_obj.clone());
        env.set(name, get_entry());

        assert_eq!(symbol.exec(&mut env), Ok(Rc::new(get_entry())))
    }

    #[test]
    fn return_as_is() {
        let mut env = Environment::new(None);
        let obj = SchemeObject::String(String::from("string"));
        let obj_ret = Ok(Rc::new(RuntimeObject::SchemeObject(obj.clone())));

        assert_eq!(obj.exec(&mut env), obj_ret);
    }
}
