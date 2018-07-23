//! Scheme standard library

use data::env::*;
use data::runtime::RuntimeObject;
use std::collections::LinkedList;
use std::rc::Rc;

/// short-hand for adding functions to an environment
/// env is the name of the `PackedEnv ` to add to
/// name is the name of the function in the environment
macro_rules! lib_funcs {
    ($env:ident, $($name:ident),*) => {{
        $(
            $env.borrow_mut().set(
                String::from(stringify!($name)),
                Rc::new(RuntimeObject::RFunc($name)),
            );
        )*
    }};
}

/// Returns an environment containing the standard library
pub fn get_std_env() -> PackedEnv {
    let env = Environment::new(None);

    //trace_macros!(true);
    lib_funcs!(env, display, id);

    env
}

// Ideally we would define these functions within lib_func! so that the function doesn't need to be duplicated. Unfortunately you can't just pass a function body into a macro because the argument names won't be defined

type Lst = LinkedList<Rc<RuntimeObject>>;
type Ret = Rc<RuntimeObject>;

fn display(lst: &Lst, _env: &PackedEnv) -> Ret {
    for arg in lst {
        print!("{:?} ", arg); // todo this shouldn't be debug printing
    }

    Rc::new(RuntimeObject::None)
}

// just for testing lib_funcs. TODO: remove this
fn id(lst: &Lst, _env: &PackedEnv) -> Ret {
    lst.front().unwrap().clone()
}
