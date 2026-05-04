use crate::core::value::Value;
use std::collections::HashMap;

fn constants() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();

    m.insert("pi", std::f64::consts::PI);
    m.insert("e", std::f64::consts::E);
    m.insert("tau", std::f64::consts::TAU);
    m.insert("phi", 1.6180339887498949);

    // Engineering constants
    m.insert("R", 8.314462618);
    m.insert("g", 9.80665);
    m.insert("Na", 6.02214076e23);
    m.insert("atm", 101325.0);

    m
}

pub fn get(name: &str) -> Option<Value> {
    constants().get(name).map(|&v| Value::new(v))
}

pub fn list() -> Vec<(String, String, f64)> {
    let mut items: Vec<(String, String, f64)> = constants()
        .iter()
        .map(|(k, v)| (k.to_string(), description(k), *v))
        .collect();
    items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    items
}

fn description(name: &str) -> String {
    match name {
        "pi" => "π = 3.14159...".to_string(),
        "e" => "Euler's number".to_string(),
        "tau" => "τ = 2π".to_string(),
        "phi" => "Golden ratio".to_string(),
        "R" => "Ideal gas constant".to_string(),
        "g" => "Gravitational acceleration".to_string(),
        "Na" => "Avogadro's number".to_string(),
        "atm" => "Standard atmosphere (Pa)".to_string(),
        _ => String::new(),
    }
}
