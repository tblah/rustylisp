//! Implements `SchemeObject::exec`

use super::env::*;
use super::scm_obj::SchemeObject;
use super::RuntimeError;
use stdlib::{get_none, get_true};

use std::collections::LinkedList;
use std::rc::Rc;

impl SchemeObject {
    /// If it is a List, execute it and return the result
    /// If it is a symbol, look it up and return the result
    /// Otherwise return as-is
    pub fn exec(&self, env: &PackedEnv) -> Result<Rc<Self>, RuntimeError> {
        match self {
            // execute code list
            SchemeObject::List(lst) => exec_codelist(&lst, env),
            // look up the symbol name in the environment
            SchemeObject::Symbol(s) => env
                .borrow_mut()
                .lookup(&s)
                .ok_or_else(|| RuntimeError::NameLookup(s.clone())),
            // return another reference to None
            SchemeObject::None => Ok(get_none()),
            // return as-is
            x => Ok(Rc::new(x.clone())),
        }
    }
}

/// helper function for `SchemeObject::exec`
fn exec_codelist(
    lst: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    // the head of the code list is the function to execute
    let scm_obj = match lst.front() {
        Some(c) => c,
        None => return Err(RuntimeError::from("Executing empty codelist")),
    };

    // get list tail - TODO immutable single linked list to avoid clone
    let tail = if lst.len() > 1 {
        lst.clone().split_off(1)
    } else {
        LinkedList::new()
    };

    // how do we call the head of the code list?
    match scm_obj {
        SchemeObject::Symbol(cmd) => {
            // is this a normal function call or a special form?
            match cmd.as_str() {
                "define" => define(&tail, env),
                "let" => scm_let(&tail, env),
                "lambda" => lambda(&tail, env),
                "if" => scm_if(&tail, env),
                _ => function_call(scm_obj, &tail, env),
            }
        }
        // We need to evaluate the code list and then exec whatever it returns
        SchemeObject::List(_) => scm_obj.exec(env)?.exec_args(&tail, env),
        // We can't call that type
        _ => Err(RuntimeError::from(format!(
            "{:?} found; function name expected",
            scm_obj
        ))),
    }
}

/// Reads in a scheme function's arguments and constructs the `SchemeObject`
fn read_scm_fn(
    name_iter: &mut Iterator<Item = &SchemeObject>,
    body: SchemeObject,
    env: &PackedEnv,
) -> Result<SchemeObject, RuntimeError> {
    // read in argument names
    let mut arg_names = Vec::new();
    for scm_obj in name_iter {
        if let SchemeObject::Symbol(arg_name) = scm_obj {
            arg_names.push(arg_name.clone());
        } else {
            return Err(RuntimeError::from("You can't call a variable that"));
        }
    }

    Ok(SchemeObject::SFunc(Box::new(body), arg_names, env.clone()))
}

/// Helper function for `exec_codelist`
/// Handles executing lambda expressions
fn lambda(
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    // two arguments: argument names and the function body
    // TODO additional arguments are more code statements for the fn body like in a let?
    if tail.len() != 2 {
        return Err(RuntimeError::from("Expected 2 arguments to lambda"));
    }

    let mut tail_iter = tail.iter();

    // first argument is the argument names
    let mut arg_names = {
        if let SchemeObject::List(lst) = tail_iter.next().unwrap() {
            lst.iter()
        } else {
            return Err(RuntimeError::from(
                "Expected the first argument to lambda to be a list",
            ));
        }
    };

    // second argument is the function body
    let body = tail_iter.next().unwrap().clone();

    // construct the SchemeObject
    Ok(Rc::new(read_scm_fn(&mut arg_names, body, env)?))
}

/// prepares to apply bindings (symbol val) for define and let
/// returns (name, value)
fn apply_biding(
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<(String, Rc<SchemeObject>), RuntimeError> {
    // check the number of arguments
    if tail.len() == 2 {
        let mut tail_iter = tail.iter(); // todo: should this be a list of Rc<>

        // first item in the binding list
        match tail_iter.next().unwrap() {
            // ordinary variable binding
            SchemeObject::Symbol(name) => {
                // return the name and the evaluated value
                let scm_obj = tail_iter.next().unwrap().clone();
                Ok((name.clone(), scm_obj.exec(env)?))
            }
            // function binding
            SchemeObject::List(lst) => {
                let mut lst_iter = lst.iter();

                // first list item is the function name
                let name = match lst_iter.next() {
                    Some(SchemeObject::Symbol(name)) => name,
                    Some(_) => return Err(RuntimeError::from("You can't name a function that")),
                    None => panic!("Empty assignment list"),
                };

                // get the function body
                let body = tail_iter.next().unwrap().clone();

                // get the function argument names and construct the `SchemeObject`
                let rt_obj = read_scm_fn(&mut lst_iter, body, env)?;
                Ok((name.clone(), Rc::new(rt_obj)))
            }
            // neither a function binding nor a symbol
            _ => Err(RuntimeError::from("You can't name a variable that")),
        }
    } else {
        Err(RuntimeError::from("Expected 2 arguments"))
    }
}

/// helper function for `exec_codelist`
/// Executes a define statement
fn define(
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    let (name, val) = apply_biding(tail, env)?;
    env.borrow_mut().set_global(name, val);

    Ok(get_none())
}

/// helper function for `exec_codelist`
/// Executes a let statement
fn scm_let(
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    let mut tail_iter = tail.iter();

    // check the number of arguments
    if tail.len() >= 2 {
        // this should be the list of lists of variables and mappings
        if let SchemeObject::List(lst) = tail_iter.next().unwrap() {
            let mut local_env = Environment::new(Some(env.clone()));

            // set all bindings in local_env
            for binding in lst {
                if let SchemeObject::List(lst) = binding {
                    let (name, val) = apply_biding(lst, env)?;
                    env.borrow_mut().set(name, val);
                } else {
                    // binding wasn't a list
                    return Err(RuntimeError::from("Let bindings should be 2 element lists"));
                }
            }

            // shrink local_env now we have finished putting it together
            local_env.borrow_mut().shrink();

            // execute the code arguments
            let mut last_ret = Err(RuntimeError::from("No let result"));

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
            Err(RuntimeError::from("You incorrect let form"))
        }
    } else {
        // incorrect number of arguments
        Err(RuntimeError::from("let should have at least 2 arguments"))
    }
}

/// helper function for `exec_codelist`
fn function_call(
    scm_obj: &SchemeObject,
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    // look up the function
    let runtime_obj = scm_obj.exec(env)?;

    // execute the function
    runtime_obj.exec_args(&tail, env)
}

fn scm_if(
    tail: &LinkedList<SchemeObject>,
    env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    if tail.len() < 2 {
        return Err(RuntimeError::from(
            "If statement needs to at least specify a condition and something to do on true",
        ));
    }

    let mut iter = tail.iter();

    // evaluate condition
    let cond = iter.next().unwrap().exec(env)?;

    let true_branch = iter.next().unwrap();

    if cond == get_true() {
        true_branch.exec(env)
    } else {
        iter.next()
            .map_or_else(|| Ok(get_none()), |so| so.exec(env))
    }
}

#[cfg(test)]
mod test {
    use ast;
    use data::env::*;
    use data::*;

    use std::collections::LinkedList;
    use std::ops::Deref;
    use std::rc::Rc;

    #[test]
    fn resolve_symbol() {
        let name = String::from("name");
        let val = String::from("value");

        let symbol = SchemeObject::sym_from(name.clone());
        let val_obj = Rc::new(SchemeObject::from(&val));

        // create an environment where "name" is mapped to "value"
        let env = Environment::new(None);
        env.borrow_mut().set(name, val_obj.clone());

        assert_eq!(symbol.exec(&env), Ok(val_obj.clone()))
    }

    #[test]
    fn return_as_is() {
        let env = Environment::new(None);
        let obj = Rc::new(SchemeObject::from("string"));
        let obj_ret = Ok(obj.clone());

        assert_eq!(obj.exec(&env), obj_ret);
    }

    /// implementation of string concatenation for use in tests
    fn cat(args: &LinkedList<Rc<SchemeObject>>, _env: &PackedEnv) -> Rc<SchemeObject> {
        let mut out = String::new();

        for arg in args {
            // concat strings or panic
            match arg.deref() {
                SchemeObject::String(s) => out += s,
                _ => panic!("Expected string arguments"),
            }
        }

        Rc::new(SchemeObject::from(out))
    }

    fn get_test_env() -> PackedEnv {
        let env = Environment::new(None);
        env.borrow_mut().set(
            String::from("cat"),
            Rc::new(SchemeObject::RFunc(String::from("cat"), cat)),
        );
        env.borrow_mut()
            .set(String::from("space"), Rc::new(SchemeObject::from(" ")));
        env.borrow_mut().shrink();
        env
    }

    fn exec_program(program: &str, expected: Vec<SchemeObject>) {
        let env = get_test_env();

        let mut chars = program.chars();
        let syntax = ast::ObjectIterator::from(&mut chars);

        for (exp, code) in expected.iter().zip(syntax) {
            let rc = code.unwrap().exec(&env).unwrap();
            assert_eq!(rc.deref(), exp);
        }
    }

    #[test]
    fn simple_codelist() {
        let program = "(cat \"Hello\" space \"world!\")";
        let expected = SchemeObject::from("Hello world!");

        exec_program(program, vec![expected])
    }

    #[test]
    fn nested_codelist() {
        let program = "(cat \"Hello\" (cat space \"world!\"))";
        let expected = SchemeObject::from("Hello world!");

        exec_program(program, vec![expected])
    }

    #[test]
    fn define() {
        let program = "(define hello \"Hello\")
             (define world \"world\")
             (cat hello space world \"!\")";
        let last = SchemeObject::from("Hello world!");
        let expected = vec![SchemeObject::None, SchemeObject::None, last];

        exec_program(program, expected)
    }

    #[test]
    fn scm_let() {
        let program = "(let ((world \"world\")
                             (hello \"Hello\"))
                            (cat hello space world \"!\"))";
        let expected = SchemeObject::from("Hello world!");

        exec_program(program, vec![expected])
    }

    #[test]
    fn shadowing_define_let() {
        let program = "(define test \"global binding\")
                       (let ((test \"local binding\"))
                            (cat test))";
        let expected = SchemeObject::from("local binding");

        exec_program(program, vec![SchemeObject::None, expected])
    }

    #[test]
    fn shadowing_let_define() {
        let program = "(let ((test \"local binding\"))
                            (define test \"global binding\")
                            (cat test))";
        let expected = SchemeObject::from("global binding");

        exec_program(program, vec![expected])
    }

    #[test]
    fn define_fn_no_args() {
        let program = "(define (f) (cat \"hello world\"))
                       (f)";
        let expected = SchemeObject::from("hello world");

        exec_program(program, vec![SchemeObject::None, expected]);
    }

    #[test]
    fn define_fn_args() {
        let program = "(define (hi person) (cat \"hi \" person))
                       (hi \"Tom\")";
        let expected = SchemeObject::from("hi Tom");

        exec_program(program, vec![SchemeObject::None, expected]);
    }

    #[test]
    fn let_fn() {
        let program = "(let ((me \"Tom\")
                             ((hi person) (cat \"hi \" person)))
                            (hi me))";
        let expected = SchemeObject::from("hi Tom");

        exec_program(program, vec![expected]);
    }

    #[test]
    fn lambda_no_args() {
        let program = "((lambda () (cat \"hello world\")))";
        let expected = SchemeObject::from("hello world");

        exec_program(program, vec![expected])
    }

    #[test]
    fn lambda_args() {
        let program = "((lambda (name) (cat \"hi \" name)) \"Tom\")";
        let expected = SchemeObject::from("hi Tom");

        exec_program(program, vec![expected]);
    }

    #[test]
    fn define_lambda() {
        let program = "(define say_hi (lambda (name)
                                              (cat \"hi \" name)))
                       (say_hi \"Tom\")";
        let expected = SchemeObject::from("hi Tom");

        exec_program(program, vec![SchemeObject::None, expected]);
    }

    #[test]
    fn higher_order_functions() {
        let program = "(define (call_with_hi fn) (lambda () (fn \"hi\")))
                     (define say_hi (call_with_hi cat))
                     (say_hi)";
        let expected = SchemeObject::from("hi");

        exec_program(
            program,
            vec![SchemeObject::None, SchemeObject::None, expected],
        );
    }

    #[test]
    fn let_over_lambda() {
        let program = "(define ret_hi
                         (let ((hi \"hi\"))
                           (lambda () hi)))
                       (ret_hi)";
        let expected = SchemeObject::from("hi");

        exec_program(program, vec![SchemeObject::None, expected]);
    }

    #[test]
    fn if_true1() {
        let program = "(if #t \"hi\")";
        let expected = SchemeObject::from("hi");
        exec_program(program, vec![expected]);
    }

    #[test]
    fn if_true2() {
        let program = "(if #t \"hi\" \"lo\")";
        let expected = SchemeObject::from("hi");
        exec_program(program, vec![expected]);
    }

    #[test]
    fn if_false1() {
        let program = "(if #f \"hi\")";
        let expected = SchemeObject::None;
        exec_program(program, vec![expected]);
    }

    #[test]
    fn if_false2() {
        let program = "(if #f \"hi\" \"lo\")";
        let expected = SchemeObject::from("lo");
        exec_program(program, vec![expected]);
    }

    #[test]
    fn if_sym() {
        let program = "(let ((sym #t))
                         (if sym \"hi\"))";
        let expected = SchemeObject::from("hi");
        exec_program(program, vec![expected])
    }
}
