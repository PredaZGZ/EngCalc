use crate::core::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.variables.iter()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
