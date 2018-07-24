//! Scheme standard library

use data::env::*;
use data::runtime::RuntimeObject;
use std::collections::LinkedList;
use std::process;
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
    lib_funcs!(env, display, exit, newline);

    env
}

// short-hand
type Lst = LinkedList<Rc<RuntimeObject>>;
type Ret = Rc<RuntimeObject>;

/// Macro to share implementation of `get_none`, `get_true` and `get_false`
/// The idea is to have only one copy of the Null, true and false objects per thread
/// The generated function returns an Rc to these objects
/// Unique true, false and None objects can result from direct construction e.g.
/// `Rc::new(RuntimeObject::from(true))`
macro_rules! const_obj {
    ($(#[$attr:meta])*, $name:ident, $init: expr) => {
        $(#[$attr])*
        /// see definition of `const_obj!`
        pub fn $name() -> Ret {
            thread_local! {
                static OBJ: Rc<RuntimeObject> = Rc::new($init);
            }

            OBJ.with(|o| o.clone())
        }
    }
}

const_obj!{
    /// share references to `RuntimeObject::None`
    , get_none, RuntimeObject::None
}

const_obj!{
    /// share references to #t
    , get_true, RuntimeObject::from(true)
}

const_obj!{
    /// share references to #f
    , get_false, RuntimeObject::from(false)
}

// Actually define standard library functions:

// Ideally we would define these functions within lib_func! so that the function doesn't need to be duplicated. Unfortunately you can't just pass a function body into a macro because the argument names won't be defined

fn display(lst: &Lst, _env: &PackedEnv) -> Ret {
    let mut iter = lst.iter();

    if let Some(obj) = iter.next() {
        print!("{}", obj); // no space
    }
    for arg in iter {
        print!(" {}", arg); // space
    }

    get_none()
}

fn exit(_lst: &Lst, _env: &PackedEnv) -> Ret {
    process::exit(0);
}

fn newline(_lst: &Lst, _env: &PackedEnv) -> Ret {
    Rc::new(RuntimeObject::from("\n"))
}
