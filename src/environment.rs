use crate::Value;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::LinkedList;

struct Scope {
    values: HashMap<String, Value>,
}

impl Scope {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    fn assign(&mut self, name: String, value: Value) -> Result<Value, ()> {
        match self.values.entry(name) {
            Entry::Occupied(mut entry) => {
                entry.insert(value.clone());
                Ok(value)
            }
            Entry::Vacant(_) => Err(()),
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }
}

pub struct Environment {
    scopes: LinkedList<Scope>,
}

impl Environment {
    pub fn new() -> Environment {
        let mut result = Self {
            scopes: LinkedList::new(),
        };
        result.push();
        result
    }

    pub fn push(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop_front().unwrap();
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.scopes.front_mut().unwrap().define(name.into(), value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        for scope in self.scopes.iter_mut() {
            match scope.assign(name.into(), value.clone()) {
                Ok(value) => return Ok(value),
                Err(_) => {}
            }
        }
        Err(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter() {
            match scope.get(name) {
                Some(value) => return Some(value),
                None => {}
            }
        }

        None
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
        e.push();
        assert_eq!(e.get("a"), Some(Value::String("outer a".to_string())));
        e.pop();
    }

    #[test]
    fn assign_outer() {
        let mut e = Environment::new();
        e.define("a", Value::String("outer a".to_string()));
        e.push();
        e.assign("a", Value::String("inner a".to_string())).unwrap();
        e.pop();
        assert_eq!(e.get("a"), Some(Value::String("inner a".to_string())));
    }

    #[test]
    fn define_inner() {
        let mut e = Environment::new();
        e.define("a", Value::String("outer a".to_string()));
        e.push();
        e.define("a", Value::String("inner a".to_string()));
        assert_eq!(e.get("a"), Some(Value::String("inner a".to_string())));
        e.pop();
        assert_eq!(e.get("a"), Some(Value::String("outer a".to_string())));
    }
}
