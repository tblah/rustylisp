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

    // get list tail - TODO immutable single linked list to avoid clone
    let tail = if lst.len() > 1 {
        lst.clone().split_off(1)
    } else {
        LinkedList::new()
    };

    if let SchemeObject::Symbol(cmd) = scm_obj {
        // is this a normal function call or a special form?
        match cmd.as_str() {
            "define" => define(&tail, env),
            "let" => panic!("TODO"),
            "lambda" => panic!("TODO"),
            "if" => panic!("TODO"),
            _ => function_call(scm_obj, &tail, env),
        }
    } else {
        Err(ParseError::SyntaxError(format!(
            "{:?} found; function name expected",
            scm_obj
        )))
    }
}

/// helper function for `exec_codelist`
fn define(
    tail: &LinkedList<SchemeObject>,
    env: &mut Environment,
) -> Result<Rc<RuntimeObject>, ParseError> {
    let mut tail_iter = tail.iter();

    // check the number of arguments
    if tail.len() == 2 {
        // set the variable in the global environment
        if let SchemeObject::Symbol(name) = tail_iter.next().unwrap() {
            let val = tail_iter.next().unwrap();
            env.set_global(name.clone(), RuntimeObject::SchemeObject(val.clone()));

            Ok(Rc::new(RuntimeObject::None))
        } else {
            // the first argument didn't look like a variable name
            Err(ParseError::SyntaxError(String::from(
                "You can't name a variable that",
            )))
        }
    } else {
        // incorrect number of arguments
        Err(ParseError::SyntaxError(String::from(
            "define should have 2 arguments",
        )))
    }
}

/// helper function for `exec_codelist`
fn function_call(
    scm_obj: &SchemeObject,
    tail: &LinkedList<SchemeObject>,
    env: &mut Environment,
) -> Result<Rc<RuntimeObject>, ParseError> {
    // look up the function
    let runtime_obj = scm_obj.exec(env)?;

    // execute the function
    runtime_obj.exec(&tail, env)
}

#[cfg(test)]
mod test {
    use ast;
    use data::env::Environment;
    use data::runtime::RuntimeObject;
    use data::*;
    use std::ops::Deref;
    use tokenise;

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

    /// implementation of string concatenation for use in tests
    fn cat(lst: &LinkedList<Rc<SchemeObject>>, _env: &mut Environment) -> Rc<RuntimeObject> {
        let mut out = String::new();

        for arg in lst {
            match arg.deref() {
                SchemeObject::String(s) => out += s,
                _ => panic!("Expected string arguments"),
            }
        }

        Rc::new(RuntimeObject::SchemeObject(SchemeObject::String(out)))
    }

    fn get_test_env() -> Environment {
        let mut env = Environment::new(None);
        env.set(String::from("cat"), RuntimeObject::RFunc(cat));
        env.set(
            String::from("space"),
            RuntimeObject::SchemeObject(SchemeObject::String(String::from(" "))),
        );
        env
    }

    fn exec_codelist(program: &str, expected: Vec<RuntimeObject>) {
        let mut env = get_test_env();

        let mut chars = program.chars();
        let tokens = tokenise::TokenIterator::new(&mut chars);
        let syntax = ast::ObjectIterator::new(tokens);

        for (exp, code) in expected.iter().zip(syntax) {
            let rc = code.unwrap().exec(&mut env).unwrap();
            // we have to mess about because we only borrow exp
            let res = Rc::try_unwrap(rc).unwrap();
            assert!(&res == exp);
        }
    }

    #[test]
    fn simple_codelist() {
        let program = "(cat \"Hello\" space \"world!\")";
        let expected =
            RuntimeObject::SchemeObject(SchemeObject::String(String::from("Hello world!")));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn nested_codelist() {
        let program = "(cat \"Hello\" (cat space \"world!\"))";
        let expected =
            RuntimeObject::SchemeObject(SchemeObject::String(String::from("Hello world!")));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn define() {
        let program = "(define hello \"Hello\")
             (define world \"world\")
             (cat hello space world \"!\")";
        let last = RuntimeObject::SchemeObject(SchemeObject::String(String::from("Hello world!")));
        let expected = vec![RuntimeObject::None, RuntimeObject::None, last];

        exec_codelist(program, expected)
    }
}
