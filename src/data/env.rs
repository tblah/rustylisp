//! Name lookup

use super::runtime::RuntimeObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Stores the environment from which variables are looked up
#[derive(Debug, PartialEq, Clone)] // we are only cloning Rc<RuntimeObject> not the actual obj
pub struct Environment {
    /// Points to the next environment in the name resolution order
    parent: Option<Rc<RefCell<Environment>>>,
    /// Mapping of variable names to objects
    names: HashMap<String, Rc<RuntimeObject>>,
}

impl Environment {
    /// Instance new Environment
    pub fn new(parent: Option<Rc<RefCell<Self>>>) -> Self {
        Self {
            parent,
            names: HashMap::new(),
        }
    }

    /// Look up variable in environment
    pub fn lookup<'a>(&'a self, name: &str) -> Option<Rc<RuntimeObject>> {
        match self.names.get(name) {
            Some(entry) => Some(entry.clone()), // just clones the Rc - no copy
            None => match &self.parent {
                Some(p) => p.borrow().lookup(name),
                None => None,
            },
        }
    }

    /// Set variable in this environment
    pub fn set(&mut self, name: String, val: RuntimeObject) {
        self.names.insert(name, Rc::new(val));
    }

    /// Set variable in the global environment
    pub fn set_global(&mut self, name: String, val: RuntimeObject) {
        match self.parent {
            None => self.set(name, val), // if self has no parent then it is global
            Some(ref mut p) => p.borrow_mut().set(name, val),
        };
    }

    /// Get parent
    pub fn get_parent(&self) -> Option<Rc<RefCell<Self>>> {
        match self.parent {
            None => None,
            Some(ref rc) => Some(rc.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use data::env::*;
    use data::SchemeObject;
    use std::rc::Rc;

    #[test]
    fn one_level() {
        let name = String::from("name");
        let g_name = String::from("global");
        let get_obj = || RuntimeObject::SchemeObject(SchemeObject::String(String::from("obj")));
        let exp_res = Some(Rc::new(get_obj()));

        let mut env = Environment::new(None);
        assert!(env.lookup(&name).is_none());
        assert!(env.lookup(&g_name).is_none());

        env.set(name.clone(), get_obj());
        assert_eq!(env.lookup(&name), exp_res);
        assert!(env.lookup(&g_name).is_none());

        env.set_global(g_name.clone(), get_obj());
        assert_eq!(env.lookup(&name), exp_res);
        assert_eq!(env.lookup(&g_name), exp_res);
    }

    #[test]
    fn two_level() {
        let name = String::from("name");
        let g_name = String::from("global");
        let get_obj = || RuntimeObject::SchemeObject(SchemeObject::String(String::from("obj")));
        let exp_res = Some(Rc::new(get_obj()));

        let g_env = Rc::new(RefCell::new(Environment::new(None)));
        let mut env = Environment::new(Some(g_env.clone()));

        assert!(env.lookup(&name).is_none());
        assert!(env.lookup(&g_name).is_none());
        assert!(g_env.borrow().lookup(&name).is_none());
        assert!(g_env.borrow().lookup(&g_name).is_none());

        env.set(name.clone(), get_obj());
        assert_eq!(env.lookup(&name), exp_res);
        assert!(env.lookup(&g_name).is_none());
        assert!(g_env.borrow().lookup(&name).is_none());
        assert!(g_env.borrow().lookup(&g_name).is_none());

        env.set_global(g_name.clone(), get_obj());

        assert_eq!(env.lookup(&name), exp_res);
        assert_eq!(env.lookup(&g_name), exp_res);
        assert!(g_env.borrow().lookup(&name).is_none());
        assert_eq!(g_env.borrow().lookup(&g_name), exp_res);
    }

    /*    /// ugly clone wrapper for these tests *only*
    impl Clone for RuntimeObject {
        fn clone(&self) -> Self {
            match self {
                RuntimeObject::SchemeObject(s) => RuntimeObject::SchemeObject(s.clone()),
                _ => panic!("I can only clone scheme objects"),
            }
        }
    }*/
}
