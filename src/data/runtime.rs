//! Defines `RuntimeObject`
//! Runtime objects are scheme objects or functions

use data::env::Environment;
use data::SchemeObject;
use ParseError;

use std::collections::LinkedList;
use std::fmt;
use std::rc::Rc;

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
/// The trait implementations are a bit fragile so read the source first
pub enum RuntimeObject {
    /// A `SchemeObject`
    SchemeObject(SchemeObject),
    /// A rust function
    RFunc(fn(&LinkedList<Rc<SchemeObject>>, &mut Environment) -> Rc<RuntimeObject>),
    /// A scheme function,
    SFunc(
        SchemeObject, // Code list
        Vec<String>,  // argument names
    ),
    /// None (for use as a function return value)
    None,
}

/// evaluates function arguments
/// helper function for `RuntimeObject::exec`
fn eval_args(
    args: &LinkedList<SchemeObject>, // TODO if this was a list of Rc<> we could avoid a clone
    env: &mut Environment,
) -> Result<LinkedList<Rc<SchemeObject>>, ParseError> {
    let mut ret = LinkedList::new();
    let results = args.iter().map(|arg| arg.exec(env));

    for res in results {
        let runtime_obj = res?;
        let scm_obj = match *runtime_obj {
            RuntimeObject::SchemeObject(ref o) => Rc::new(o.clone()),
            _ => panic!("Trying to pass a runtime object as an argument!"),
        };
        ret.push_back(scm_obj);
    }

    Ok(ret)
}

impl RuntimeObject {
    /// evaluates a runtime object
    pub fn exec(
        &self,
        args: &LinkedList<SchemeObject>,
        env: &mut Environment,
    ) -> Result<Rc<Self>, ParseError> {
        match self {
            RuntimeObject::SchemeObject(o) => o.exec(env),
            RuntimeObject::RFunc(ref f) => {
                let evaled_args = eval_args(args, env)?;
                Ok(f(&evaled_args, env))
            }
            RuntimeObject::SFunc(_, _) => panic!("TODO - evaluating scheme functions"),
            RuntimeObject::None => Ok(Rc::new(RuntimeObject::None)),
        }
    }
}

impl PartialEq for RuntimeObject {
    fn eq(&self, other: &Self) -> bool {
        use self::RuntimeObject::{None, SFunc, SchemeObject};
        match (self, other) {
            (SchemeObject(s1), SchemeObject(s2)) => s1 == s2,
            (SFunc(s1, v1), SFunc(s2, v2)) => s1 == s2 && v1 == v2,
            (None, None) => true,
            _ => false, // different types or rust functions
        }
    }
}

impl fmt::Debug for RuntimeObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeObject::SchemeObject(s) => s.fmt(f),
            RuntimeObject::SFunc(_, names) => write!(f, "Scheme function with args {:?}", names),
            RuntimeObject::RFunc(_) => write!(f, "Built-in function"),
            RuntimeObject::None => write!(f, "None"),
        }
    }
}
