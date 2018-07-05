//! Name lookup

use super::runtime::RuntimeObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
/// `Environment`s are used packed in `Rc<RefCell<Environment>>`
pub type PackedEnv = Rc<RefCell<Environment>>;

/// Stores the environment from which variables are looked up
#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    /// Points to the next environment in the name resolution order
    parent: Option<PackedEnv>,
    /// Mapping of variable names to objects
    names: HashMap<String, Rc<RuntimeObject>>,
}

impl Environment {
    /// Instance new Environment
    pub fn new(parent: Option<Rc<RefCell<Self>>>) -> PackedEnv {
        let env = Self {
            parent,
            names: HashMap::new(),
        };

        Rc::new(RefCell::new(env))
    }

    /// Look up variable in environment
    pub fn lookup(&self, name: &str) -> Option<Rc<RuntimeObject>> {
        match self.names.get(name) {
            Some(entry) => Some(entry.clone()), // just clones the Rc - no copy
            None => match &self.parent {
                Some(p) => p.borrow().lookup(name),
                None => None,
            },
        }
    }

    /// Set variable in this environment
    pub fn set(&mut self, name: String, val: Rc<RuntimeObject>) {
        self.names.insert(name, val);
    }

    /// Set variable in the global environment and change the local environment to avoid shadowing
    pub fn set_global(&mut self, name: String, val: Rc<RuntimeObject>) {
        // we don't want any local variables to shadow the new definition
        self.names.remove(&name.clone());
        self.set_global_priv(name, val);
    }

    /// Only do the traversal to the global environment. Don't touch the local environment
    fn set_global_priv(&mut self, name: String, val: Rc<RuntimeObject>) {
        match self.parent {
            None => self.set(name, val), // if self has no parent then it is global
            Some(ref mut p) => {
                p.borrow_mut().set_global_priv(name, val);
            }
        };
    }

    /// Get parent
    pub fn get_parent(&self) -> Option<Rc<RefCell<Self>>> {
        match self.parent {
            None => None,
            Some(ref rc) => Some(rc.clone()),
        }
    }

    /// Set the parent
    pub fn set_parent(&mut self, parent: Option<Rc<RefCell<Self>>>) {
        self.parent = parent;
    }

    /// shrink
    pub fn shrink(&mut self) {
        self.names.shrink_to_fit()
    }
}

#[cfg(test)]
mod tests {
    use data::env::*;
    use std::rc::Rc;

    #[test]
    fn one_level() {
        let name = String::from("name");
        let g_name = String::from("global");
        let get_obj = || Rc::new(RuntimeObject::from("obj"));
        let exp_res = Some(get_obj());

        let env = Environment::new(None);
        assert!(env.borrow().lookup(&name).is_none());
        assert!(env.borrow().lookup(&g_name).is_none());

        env.borrow_mut().set(name.clone(), get_obj());
        assert_eq!(env.borrow().lookup(&name), exp_res);
        assert!(env.borrow().lookup(&g_name).is_none());

        env.borrow_mut().set_global(g_name.clone(), get_obj());
        assert_eq!(env.borrow().lookup(&name), exp_res);
        assert_eq!(env.borrow().lookup(&g_name), exp_res);
    }

    #[test]
    fn two_level() {
        let name = String::from("name");
        let g_name = String::from("global");
        let get_obj = || Rc::new(RuntimeObject::from("obj"));
        let exp_res = Some(get_obj());

        let g_env = Environment::new(None);
        let env = Environment::new(Some(g_env.clone()));

        assert!(env.borrow().lookup(&name).is_none());
        assert!(env.borrow().lookup(&g_name).is_none());
        assert!(g_env.borrow().lookup(&name).is_none());
        assert!(g_env.borrow().lookup(&g_name).is_none());

        env.borrow_mut().set(name.clone(), get_obj());
        assert_eq!(env.borrow().lookup(&name), exp_res);
        assert!(env.borrow().lookup(&g_name).is_none());
        assert!(g_env.borrow().lookup(&name).is_none());
        assert!(g_env.borrow().lookup(&g_name).is_none());

        env.borrow_mut().set_global(g_name.clone(), get_obj());

        assert_eq!(env.borrow().lookup(&name), exp_res);
        assert_eq!(env.borrow().lookup(&g_name), exp_res);
        assert!(g_env.borrow().lookup(&name).is_none());
        assert_eq!(g_env.borrow().lookup(&g_name), exp_res);
    }
}
