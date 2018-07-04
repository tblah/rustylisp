//! Defines `RuntimeObject`
//! Runtime objects are scheme objects or functions

use data::env::Environment;
use data::SchemeObject;
use ParseError;

use std::cell::RefCell;
use std::collections::LinkedList;
use std::fmt;
use std::rc::Rc;

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
/// A scheme object which can result from the execution of scheme code
/// The trait implementations are a bit fragile so read the source first
pub enum RuntimeObject {
    /// A `SchemeObject`
    SchemeObject(Rc<SchemeObject>),
    /// A rust function
    RFunc(fn(&LinkedList<Rc<RuntimeObject>>, &Rc<RefCell<Environment>>) -> Rc<RuntimeObject>),
    /// A scheme function,
    SFunc(
        SchemeObject, // Code list
        Vec<String>,  // argument names
        // closure environment (the environment in use when the function was defined)
        Rc<RefCell<Environment>>,
    ),
    /// None (for use as a function return value)
    None,
}

/// evaluates function arguments
/// helper function for `RuntimeObject::exec`
fn eval_args(
    args: &LinkedList<SchemeObject>, // TODO if this was a list of Rc<> we could avoid a clone
    env: &Rc<RefCell<Environment>>,
) -> Result<LinkedList<Rc<RuntimeObject>>, ParseError> {
    let mut ret = LinkedList::new();

    for arg in args {
        ret.push_back(arg.exec(env)?);
    }

    Ok(ret)
}

impl RuntimeObject {
    /// evaluates a runtime object
    pub fn exec(
        &self,
        args: &LinkedList<SchemeObject>,
        env: &Rc<RefCell<Environment>>,
    ) -> Result<Rc<Self>, ParseError> {
        match self {
            RuntimeObject::SchemeObject(o) => o.exec(env),
            RuntimeObject::RFunc(ref f) => {
                let evaled_args = eval_args(args, env)?;
                // call the function
                Ok(f(&evaled_args, env))
            }
            RuntimeObject::SFunc(code_lst, arg_names, local_env) => {
                exec_sfunc(code_lst, arg_names, args, local_env)
            }
            RuntimeObject::None => Ok(Rc::new(RuntimeObject::None)),
        }
    }
}

/// Helper function for `RuntimeObject::exec`
/// Evaluates scheme functions (`RuntimeObject::SFunc`)
fn exec_sfunc(
    code_list: &SchemeObject,
    arg_names: &[String],
    func_args: &LinkedList<SchemeObject>,
    g_env: &Rc<RefCell<Environment>>,
) -> Result<Rc<RuntimeObject>, ParseError> {
    // did we get the correct number of arguments
    // TODO variable number of arguments
    if func_args.len() != arg_names.len() {
        return Err(ParseError::SyntaxError(format!(
            "Expected {} arguments, got {}",
            arg_names.len(),
            func_args.len()
        )));
    }

    // evaluate arguments
    let evaled_args = eval_args(func_args, g_env)?;

    // add arguments to local environment
    let local_env = Environment::new(Some(g_env.clone()));
    for (name, arg) in arg_names.iter().zip(evaled_args) {
        local_env.borrow_mut().set(name.clone(), arg);
    }
    local_env.borrow_mut().shrink();

    code_list.exec(&local_env)
}

impl PartialEq for RuntimeObject {
    fn eq(&self, other: &Self) -> bool {
        use self::RuntimeObject::{None, SFunc, SchemeObject};
        match (self, other) {
            (SchemeObject(s1), SchemeObject(s2)) => s1 == s2,
            (SFunc(s1, v1, e1), SFunc(s2, v2, e2)) => s1 == s2 && v1 == v2 && e1 == e2,
            (None, None) => true,
            _ => false, // different types or rust functions
        }
    }
}

impl fmt::Debug for RuntimeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeObject::SchemeObject(s) => s.fmt(f),
            RuntimeObject::SFunc(_, names, _) => write!(f, "Scheme function with args {:?}", names),
            RuntimeObject::RFunc(_) => write!(f, "Built-in function"),
            RuntimeObject::None => write!(f, "None"),
        }
    }
}
