use crate::core::ast::Expr;
use crate::core::value::Value;
use std::collections::HashMap;

/// A user-defined function
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.variables.iter()
    }

    #[allow(dead_code)]
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    // Function-related methods
    pub fn get_function(&self, name: &str) -> Option<&UserFunction> {
        self.functions.get(name)
    }

    pub fn set_function(&mut self, func: UserFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    #[allow(dead_code)]
    pub fn remove_function(&mut self, name: &str) -> Option<UserFunction> {
        self.functions.remove(name)
    }

    #[allow(dead_code)]
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn iter_functions(&self) -> impl Iterator<Item = (&String, &UserFunction)> {
        self.functions.iter()
    }

    /// Clear all variables and functions
    pub fn clear(&mut self) {
        self.variables.clear();
        self.functions.clear();
    }

    /// Copy all variables and functions from another environment
    pub fn copy_from(&mut self, other: &Environment) {
        for (name, value) in other.iter() {
            self.variables.insert(name.clone(), value.clone());
        }
        for (name, func) in other.iter_functions() {
            self.functions.insert(name.clone(), func.clone());
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
