//! Implements `exec_args` for `SchemeObject`

use data::env::*;
use data::scm_obj::SchemeObject;
use data::RuntimeError;

use std::collections::LinkedList;
use std::rc::Rc;

impl SchemeObject {
    /// evaluates a `SchemeObject` using a set of arguments
    pub fn exec_args(
        &self,
        args: &LinkedList<Self>,
        env: &PackedEnv,
    ) -> Result<Rc<Self>, RuntimeError> {
        use self::SchemeObject::*;

        match self {
            RFunc(_, ref f) => {
                let evaled_args = eval_args(args, env)?;
                // call the function
                Ok(f(&evaled_args, env))
            }
            SFunc(code_lst, arg_names, local_env) => {
                exec_sfunc(code_lst, arg_names, args, local_env)
            }
            o => o.exec(env),
        }
    }
}

/// evaluates function arguments
/// helper function for `SchemeObject::exec_args`
fn eval_args(
    args: &LinkedList<SchemeObject>, // TODO if this was a list of Rc<> we could avoid a clone
    env: &PackedEnv,
) -> Result<LinkedList<Rc<SchemeObject>>, RuntimeError> {
    let mut ret = LinkedList::new();

    for arg in args {
        ret.push_back(arg.exec(env)?);
    }

    Ok(ret)
}

/// Helper function for `SchemeObject::exec_args`
/// Evaluates scheme functions (`SchemeObject::SFunc`)
fn exec_sfunc(
    code_list: &SchemeObject,
    arg_names: &[String],
    func_args: &LinkedList<SchemeObject>,
    g_env: &PackedEnv,
) -> Result<Rc<SchemeObject>, RuntimeError> {
    // did we get the correct number of arguments
    // TODO variable number of arguments
    if func_args.len() != arg_names.len() {
        return Err(RuntimeError::from(format!(
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
