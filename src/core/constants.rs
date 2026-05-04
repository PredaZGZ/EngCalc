use crate::core::value::Value;
use std::collections::HashMap;

pub struct ConstantInfo {
    pub name: &'static str,
    pub value: f64,
    pub description: &'static str,
    pub units: &'static str,
}

fn constants_data() -> Vec<ConstantInfo> {
    vec![
        ConstantInfo {
            name: "pi",
            value: std::f64::consts::PI,
            description: "Circle constant",
            units: "dimensionless",
        },
        ConstantInfo {
            name: "e",
            value: std::f64::consts::E,
            description: "Euler's number",
            units: "dimensionless",
        },
        ConstantInfo {
            name: "tau",
            value: std::f64::consts::TAU,
            description: "τ = 2π",
            units: "dimensionless",
        },
        ConstantInfo {
            name: "phi",
            value: 1.6180339887498949,
            description: "Golden ratio",
            units: "dimensionless",
        },
        ConstantInfo {
            name: "R",
            value: 8.314462618,
            description: "Ideal gas constant",
            units: "J/(mol·K)",
        },
        ConstantInfo {
            name: "g",
            value: 9.80665,
            description: "Gravitational accel.",
            units: "m/s²",
        },
        ConstantInfo {
            name: "Na",
            value: 6.02214076e23,
            description: "Avogadro's number",
            units: "mol⁻¹",
        },
        ConstantInfo {
            name: "atm",
            value: 101325.0,
            description: "Standard atmosphere",
            units: "Pa",
        },
    ]
}

fn constants_map() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    for c in constants_data() {
        m.insert(c.name, c.value);
    }
    m
}

pub fn get(name: &str) -> Option<Value> {
    constants_map().get(name).map(|&v| Value::new(v))
}

pub fn list() -> Vec<ConstantInfo> {
    let mut items = constants_data();
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}

pub fn search(query: &str) -> Vec<ConstantInfo> {
    if query.is_empty() {
        return constants_data();
    }
    let q = query.to_lowercase();
    constants_data()
        .into_iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&q)
                || c.description.to_lowercase().contains(&q)
                || c.units.to_lowercase().contains(&q)
        })
        .collect()
}
