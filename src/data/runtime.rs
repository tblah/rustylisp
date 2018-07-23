//! Defines `RuntimeObject`
//! Runtime objects are scheme objects or functions

use data::env::*;
use data::scm_static::SchemeObject;
use stdlib::get_none;
use ParseError;

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
    RFunc(fn(&LinkedList<Rc<RuntimeObject>>, &PackedEnv) -> Rc<RuntimeObject>),
    /// A scheme function,
    SFunc(
        SchemeObject, // Code list
        Vec<String>,  // argument names
        // closure environment (the environment in use when the function was defined)
        PackedEnv,
    ),
    /// None (for use as a function return value)
    None,
}

impl fmt::Display for RuntimeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RuntimeObject::SchemeObject(ref so) => so.fmt(f),
            ref other => write!(f, "{:?}", other),
        }
    }
}

/// Implement all `From<T>` which are implemented for `SchemeObject`
impl<T> From<T> for RuntimeObject
where
    T: Into<SchemeObject>,
{
    fn from(x: T) -> Self {
        RuntimeObject::SchemeObject(Rc::new(x.into()))
    }
}

/// Creates a `RuntimeObject::SchemeObject`
impl From<Rc<SchemeObject>> for RuntimeObject {
    fn from(rc: Rc<SchemeObject>) -> Self {
        RuntimeObject::SchemeObject(rc)
    }
}

/// evaluates function arguments
/// helper function for `RuntimeObject::exec`
fn eval_args(
    args: &LinkedList<SchemeObject>, // TODO if this was a list of Rc<> we could avoid a clone
    env: &PackedEnv,
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
        env: &PackedEnv,
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
            RuntimeObject::None => Ok(get_none()),
        }
    }
}

/// Helper function for `RuntimeObject::exec`
/// Evaluates scheme functions (`RuntimeObject::SFunc`)
fn exec_sfunc(
    code_list: &SchemeObject,
    arg_names: &[String],
    func_args: &LinkedList<SchemeObject>,
    g_env: &PackedEnv,
) -> Result<Rc<RuntimeObject>, ParseError> {
    // did we get the correct number of arguments
    // TODO variable number of arguments
    if func_args.len() != arg_names.len() {
        return Err(ParseError::from(format!(
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

#[cfg(test)]
mod test {
    use data::runtime::RuntimeObject;
    use data::scm_static::SchemeObject;

    #[test]
    fn from_trait() {
        let rt_str = RuntimeObject::from("str");
        let expected = RuntimeObject::from(SchemeObject::from("str"));
        assert_eq!(rt_str, expected);

        let string = String::from("string");
        let rt_string = RuntimeObject::from(string.clone());
        let expected = RuntimeObject::from(SchemeObject::from(string));
        assert_eq!(rt_string, expected);

        let rt_bool = RuntimeObject::from(true);
        let expected = RuntimeObject::from(SchemeObject::from(true));
        assert_eq!(rt_bool, expected);
    }
}
