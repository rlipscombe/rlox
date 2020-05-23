use crate::environment::Environment;
use crate::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn register_globals(environment: &mut Environment) {
    let clock = Value::NativeFunction {
        name: "clock".to_string(),
        arity: 0,
        fun: |_argv| {
            let now = SystemTime::now();
            Value::Number(now.duration_since(UNIX_EPOCH).unwrap().as_millis() as f64)
        },
    };
    environment.define("clock", clock);
}