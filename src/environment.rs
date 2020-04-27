use crate::Value;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
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
            },
            Entry::Vacant(_) => Err(())
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }
}

pub struct Environment {
    scopes: LinkedList<Scope>
}

impl Environment {
    pub fn new() -> Environment {
        Self {
            scopes: LinkedList::new()
        }
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
