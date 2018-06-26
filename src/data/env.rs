//! Name lookup

use super::SchemeObject;
use std::collections::HashMap;
use std::rc::Rc;

/// Stores the environment from which variables are looked up
#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    /// Points to the next environment in the name resolution order
    parent: Option<Rc<Environment>>,
    /// Mapping of variable names to entries
    names: HashMap<String, EnvironmentEntry>,
}

/// Stores one item bound to a variable
#[derive(Debug, PartialEq, Clone)]
pub struct EnvironmentEntry {
    /// The environment (for closures)
    env: Option<Rc<Environment>>,
    /// The thing being pointed to
    obj: SchemeObject,
}

impl EnvironmentEntry {
    /// Instance new EnvironmentEntry
    pub fn new(obj: SchemeObject, env: Option<Rc<Environment>>) -> Self {
        Self { env, obj }
    }
}

impl Environment {
    /// Instance new Environment
    pub fn new(parent: Option<Rc<Self>>) -> Self {
        Self {
            parent,
            names: HashMap::new(),
        }
    }

    /// Look up variable in environment
    pub fn lookup<'a>(&'a self, name: &str) -> Option<&'a EnvironmentEntry> {
        match self.names.get(name) {
            Some(entry) => Some(entry),
            None => match &self.parent {
                Some(p) => p.lookup(name),
                None => None,
            },
        }
    }

    /// Set variable in environment
    pub fn set(&mut self, name: String, val: EnvironmentEntry) {
        self.names.insert(name, val);
    }

    /// Set variable in global environment
    pub fn set_global(&mut self, name: String, val: EnvironmentEntry) {
        let err_msg = "env.rs: Environment::set_global - could not borrow parent env mutably";
        match self.parent {
            None => self.set(name, val), // if self has no parent then it is global
            Some(ref mut p) => Rc::get_mut(p).expect(err_msg).set(name, val),
        };
    }

    /// Get parent
    pub fn get_parent(&self) -> Option<Rc<Self>> {
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
        let obj = SchemeObject::String(String::from("obj"));
        let val = EnvironmentEntry::new(obj.clone(), None);
        let g_val = EnvironmentEntry::new(obj, None);

        let mut env = Environment::new(None);
        assert!(env.lookup(&name).is_none());
        assert!(env.lookup(&g_name).is_none());

        env.set(name.clone(), val.clone());
        assert_eq!(env.lookup(&name), Some(&val));
        assert!(env.lookup(&g_name).is_none());

        env.set_global(g_name.clone(), g_val.clone());
        assert_eq!(env.lookup(&name), Some(&val));
        assert_eq!(env.lookup(&g_name), Some(&g_val));
    }

    #[test]
    fn two_level() {
        let name = String::from("name");
        let g_name = String::from("global");
        let obj = SchemeObject::String(String::from("obj"));
        let val = EnvironmentEntry::new(obj.clone(), None);
        let g_val = EnvironmentEntry::new(obj, None);

        let g_env = Rc::new(Environment::new(None));
        let mut env = Environment::new(Some(g_env.clone()));

        assert!(env.lookup(&name).is_none());
        assert!(env.lookup(&g_name).is_none());
        assert!(g_env.lookup(&name).is_none());
        assert!(g_env.lookup(&g_name).is_none());

        env.set(name.clone(), val.clone());
        assert_eq!(env.lookup(&name), Some(&val));
        assert!(env.lookup(&g_name).is_none());
        assert!(g_env.lookup(&name).is_none());
        assert!(g_env.lookup(&g_name).is_none());

        drop(g_env); // set_global needs mutable access to env.parent
        env.set_global(g_name.clone(), g_val.clone());
        // now mutable access is done we are allowed a ref
        let g_env = env.get_parent().unwrap();

        assert_eq!(env.lookup(&name), Some(&val));
        assert_eq!(env.lookup(&g_name), Some(&g_val));
        assert!(g_env.lookup(&name).is_none());
        assert_eq!(g_env.lookup(&g_name), Some(&g_val));
    }
}
