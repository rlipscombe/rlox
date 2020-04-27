use std::collections::HashMap;
use crate::Value;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }
    pub fn assign(&mut self, name: &str, value: Value) -> Result<Value, ()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            Ok(value)
        } else {
            Err(())
        }
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).and_then(|v| Some(v.clone()))
    }
}
