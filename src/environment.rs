use crate::Value;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;

struct Scope {
    values: HashMap<String, Value>,
    enclosing: Option<Environment>,
}

#[derive(Clone)]
pub struct Environment {
    scope: Rc<RefCell<Scope>>,
}

impl Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // TODO: Actually emit something here.
        f.debug_struct("Environment").finish()
    }
}

impl PartialEq for Environment {
    fn eq(&self, _other: &Environment) -> bool {
        // Environments are never equivalent.
        false
    }
}

impl Scope {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn with_enclosing(enclosing: Environment) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        match self.values.entry(name.to_string()) {
            Entry::Occupied(mut entry) => {
                entry.insert(value.clone());
                Ok(value)
            }
            Entry::Vacant(_) => match &mut self.enclosing {
                None => Err(()),
                Some(e) => e.assign(name, value),
            },
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(v) => Some(v.clone()),
            None => match &self.enclosing {
                None => None,
                Some(e) => e.get(name),
            },
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
        }
    }

    pub fn with_enclosing(enclosing: &Environment) -> Self {
        Self {
            scope: Scope::with_enclosing(enclosing.clone()),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.scope.borrow_mut().define(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        self.scope.borrow_mut().assign(name, value)
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.scope.borrow().get(name)
    }
}

#[cfg(test)]
mod test {
    use crate::Environment;
    use crate::Value;

    #[test]
    fn define_then_get() {
        let mut e = Environment::new();
        e.define("meaning", Value::Number(42.0));
        assert_eq!(e.get("meaning"), Some(Value::Number(42.0)));
    }

    #[test]
    fn define_then_assign_then_get() {
        let mut e = Environment::new();
        e.define("there_yet", Value::Boolean(false));
        assert_eq!(e.get("there_yet"), Some(Value::Boolean(false)));

        assert!(e.assign("there_yet", Value::Boolean(false)).is_ok());
        assert_eq!(e.get("there_yet"), Some(Value::Boolean(false)));

        assert!(e.assign("there_yet", Value::Boolean(true)).is_ok());
        assert_eq!(e.get("there_yet"), Some(Value::Boolean(true)));
    }

    #[test]
    fn get_unknown() {
        let e = Environment::new();
        assert_eq!(e.get("who"), None);
    }

    #[test]
    fn assign_unknown() {
        let mut e = Environment::new();
        assert!(e.assign("what", Value::Boolean(false)).is_err());
    }

    #[test]
    fn get_outer() {
        let mut e = Environment::new();
        e.define("a", Value::String("outer a".to_string()));
        {
            let e = Environment::with_enclosing(&e);
            assert_eq!(e.get("a"), Some(Value::String("outer a".to_string())));
        }
    }

    #[test]
    fn assign_outer() {
        let mut e = Environment::new();
        e.define("a", Value::String("outer a".to_string()));
        {
            let mut e = Environment::with_enclosing(&e);
            e.assign("a", Value::String("inner a".to_string())).unwrap();
        }
        assert_eq!(e.get("a"), Some(Value::String("inner a".to_string())));
    }

    #[test]
    fn define_inner() {
        let mut e = Environment::new();
        e.define("a", Value::String("outer a".to_string()));
        {
            let mut e = Environment::with_enclosing(&e);
            e.define("a", Value::String("inner a".to_string()));
            assert_eq!(e.get("a"), Some(Value::String("inner a".to_string())));
        }
        assert_eq!(e.get("a"), Some(Value::String("outer a".to_string())));
    }
}
