//! Scheme standard library

use data::env::Environment;
use data::runtime::RuntimeObject;
use data::SchemeObject;
use std::collections::LinkedList;
use std::rc::Rc;

/// Returns an environment containing the standard library
pub fn get_std_env() -> Environment {
    let mut env = Environment::new(None);

    env.set(String::from("disp"), RuntimeObject::RFunc(disp));

    env
}

fn disp(lst: &LinkedList<Rc<SchemeObject>>, _env: &mut Environment) -> Rc<RuntimeObject> {
    for arg in lst {
        print!("{:?} ", arg); // todo this shouldn't be debug printing
    }

    Rc::new(RuntimeObject::None)
}
