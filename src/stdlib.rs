//! Scheme standard library

use data::env::Environment;
use data::runtime::RuntimeObject;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

/// Returns an environment containing the standard library
pub fn get_std_env() -> Rc<RefCell<Environment>> {
    let env = Environment::new(None);

    env.borrow_mut()
        .set(String::from("disp"), Rc::new(RuntimeObject::RFunc(disp)));

    env
}

fn disp(lst: &LinkedList<Rc<RuntimeObject>>, _env: &Rc<RefCell<Environment>>) -> Rc<RuntimeObject> {
    for arg in lst {
        print!("{:?} ", arg); // todo this shouldn't be debug printing
    }

    Rc::new(RuntimeObject::None)
}
