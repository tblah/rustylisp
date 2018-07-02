//! Data model

pub mod env;
pub mod runtime;

use self::env::*;
use self::runtime::RuntimeObject;
use super::ParseError;
use std::cell::RefCell;
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
    pub fn exec(&self, env: &Rc<RefCell<Environment>>) -> Result<Rc<RuntimeObject>, ParseError> {
        match self {
            // look up the symbol name in the environment
            SchemeObject::Symbol(s) => env
                .borrow_mut()
                .lookup(&s)
                .ok_or_else(|| ParseError::NameLookup(s.clone())),
            SchemeObject::CodeList(lst) => exec_codelist(&lst, env),
            x => Ok(Rc::new(RuntimeObject::SchemeObject(Rc::new(x.clone())))),
        }
    }
}

/// helper function for `SchemeObject::exec`
fn exec_codelist(
    lst: &LinkedList<SchemeObject>,
    env: &Rc<RefCell<Environment>>,
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
            "define" => define(&tail, &env),
            "let" => scm_let(&tail, &env),
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

/// prepares to apply bindings (symbol val) for define and let
/// returns (name, value)
fn apply_biding(tail: &LinkedList<SchemeObject>) -> Result<(String, RuntimeObject), ParseError> {
    // check the number of arguments
    if tail.len() == 2 {
        let mut tail_iter = tail.iter(); // todo: should this be a list of Rc<>

        match tail_iter.next().unwrap() {
            // ordinary variable binding
            SchemeObject::Symbol(name) => {
                // return the name and the value
                let scm_obj = tail_iter.next().unwrap().clone();
                Ok((name.clone(), RuntimeObject::SchemeObject(Rc::new(scm_obj))))
            }
            // function binding
            SchemeObject::CodeList(lst) => {
                let mut lst_iter = lst.iter();

                // first list item is the function name
                let name = match lst_iter.next() {
                    Some(SchemeObject::Symbol(name)) => name,
                    Some(_) => {
                        return Err(ParseError::SyntaxError(String::from(
                            "You can't name a function that",
                        )))
                    }
                    None => panic!("Empty let assignment list"),
                };

                // subsequent arguments are argument names for the function
                let mut arg_names = Vec::new();
                for scm_obj in lst_iter {
                    match scm_obj {
                        SchemeObject::Symbol(arg_name) => arg_names.push(arg_name.clone()),
                        _ => {
                            return Err(ParseError::SyntaxError(String::from(
                                "You can't call a variable that",
                            )))
                        }
                    }
                }

                // get the function body
                let body = tail_iter.next().unwrap().clone();

                let rt_obj = RuntimeObject::SFunc(body, arg_names);
                Ok((name.clone(), rt_obj))
            }
            // error
            _ => Err(ParseError::SyntaxError(String::from(
                "You can't name a variable that",
            ))),
        }
    } else {
        Err(ParseError::SyntaxError(String::from(
            "Expected 2 arguments",
        )))
    }
}

/// helper function for `exec_codelist`
fn define(
    tail: &LinkedList<SchemeObject>,
    env: &Rc<RefCell<Environment>>,
) -> Result<Rc<RuntimeObject>, ParseError> {
    let (name, val) = apply_biding(tail)?;
    env.borrow_mut().set_global(name, val);

    Ok(Rc::new(RuntimeObject::None))
}

/// helper function for `exec_codelist`
fn scm_let(
    tail: &LinkedList<SchemeObject>,
    env: &Rc<RefCell<Environment>>,
) -> Result<Rc<RuntimeObject>, ParseError> {
    let mut tail_iter = tail.iter();

    // check the number of arguments
    if tail.len() >= 2 {
        // this should be the list of lists of variables and mappings
        if let SchemeObject::CodeList(lst) = tail_iter.next().unwrap() {
            let mut local_env = Environment::new(Some(env.clone()));

            // set all bindings in local_env
            for binding in lst {
                if let SchemeObject::CodeList(lst) = binding {
                    let (name, val) = apply_biding(lst)?;
                    env.borrow_mut().set(name, val);
                } else {
                    // binding wasn't a list
                    return Err(ParseError::SyntaxError(String::from(
                        "Let bindings should be 2 element lists",
                    )));
                }
            }
            // execute the code arguments
            let mut last_ret = Err(ParseError::SyntaxError(String::from("No let result")));

            for code in tail_iter {
                last_ret = code.exec(&local_env);

                // return early if we encounter an error
                if last_ret.is_err() {
                    return last_ret;
                }
            }

            // return the result of the last operation
            last_ret
        } else {
            // the first argument didn't look right
            Err(ParseError::SyntaxError(String::from(
                "You incorrect let form",
            )))
        }
    } else {
        // incorrect number of arguments
        Err(ParseError::SyntaxError(String::from(
            "let should have at least 2 arguments",
        )))
    }
}

/// helper function for `exec_codelist`
fn function_call(
    scm_obj: &SchemeObject,
    tail: &LinkedList<SchemeObject>,
    env: &Rc<RefCell<Environment>>,
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
    use std::rc::Rc;
    use tokenise;

    #[test]
    fn resolve_symbol() {
        let name = String::from("name");
        let val = String::from("value");

        let symbol = SchemeObject::Symbol(name.clone());
        let val_obj = Rc::new(SchemeObject::String(val.clone()));

        // create an environment where "name" is mapped to "value"
        let env = Environment::new(None);
        let get_entry = || RuntimeObject::SchemeObject(val_obj.clone());
        env.borrow_mut().set(name, get_entry());

        assert_eq!(symbol.exec(&env), Ok(Rc::new(get_entry())))
    }

    #[test]
    fn return_as_is() {
        let env = Environment::new(None);
        let obj = Rc::new(SchemeObject::String(String::from("string")));
        let obj_ret = Ok(Rc::new(RuntimeObject::SchemeObject(obj.clone())));

        assert_eq!(obj.exec(&env), obj_ret);
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

        Rc::new(RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(
            out,
        ))))
    }

    fn get_test_env() -> Rc<RefCell<Environment>> {
        let env = Environment::new(None);
        env.borrow_mut()
            .set(String::from("cat"), RuntimeObject::RFunc(cat));
        env.borrow_mut().set(
            String::from("space"),
            RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(" ")))),
        );
        env
    }

    fn exec_codelist(program: &str, expected: Vec<RuntimeObject>) {
        let env = get_test_env();

        let mut chars = program.chars();
        let tokens = tokenise::TokenIterator::new(&mut chars);
        let syntax = ast::ObjectIterator::new(tokens);

        for (exp, code) in expected.iter().zip(syntax) {
            let rc = code.unwrap().exec(&env).unwrap();
            // we have to mess about because we only borrow exp
            let res = Rc::try_unwrap(rc).unwrap();
            assert!(&res == exp);
        }
    }

    #[test]
    fn simple_codelist() {
        let program = "(cat \"Hello\" space \"world!\")";
        let expected = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "Hello world!",
        ))));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn nested_codelist() {
        let program = "(cat \"Hello\" (cat space \"world!\"))";
        let expected = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "Hello world!",
        ))));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn define() {
        let program = "(define hello \"Hello\")
             (define world \"world\")
             (cat hello space world \"!\")";
        let last = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "Hello world!",
        ))));
        let expected = vec![RuntimeObject::None, RuntimeObject::None, last];

        exec_codelist(program, expected)
    }

    #[test]
    fn scm_let() {
        let program = "(let ((world \"world\")
                             (hello \"Hello\"))
                            (cat hello space world \"!\"))";
        let expected = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "Hello world!",
        ))));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn shadowing_define_let() {
        let program = "(define test \"global binding\")
                       (let ((test \"local binding\"))
                            (cat test))";
        let expected = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "local binding",
        ))));

        exec_codelist(program, vec![RuntimeObject::None, expected])
    }

    #[test]
    fn shadowing_let_define() {
        let program = "(let ((test \"local binding\"))
                            (define test \"global binding\")
                            (cat test))";
        let expected = RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from(
            "global binding",
        ))));

        exec_codelist(program, vec![expected])
    }

    #[test]
    fn define_fn_no_args() {
        let program = "(define (f) (cat \"hello world\"))
                       (f)";
        let expected =
            RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from("hello world"))));

        exec_codelist(program, vec![RuntimeObject::None, expected]);
    }

    #[test]
    fn define_fn_args() {
        let program = "(define (hi person) (cat \"hi \" person))
                       (hi \"Tom\")";
        let expected =
            RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from("hi Tom"))));

        exec_codelist(program, vec![RuntimeObject::None, expected]);
    }

    #[test]
    fn let_fn() {
        let program = "(let ((me \"Tom\")
                             ((hi person) (cat \"hi \" person)))
                            (hi me))";
        let expected =
            RuntimeObject::SchemeObject(Rc::new(SchemeObject::String(String::from("hi Tom"))));

        exec_codelist(program, vec![expected]);
    }
}
