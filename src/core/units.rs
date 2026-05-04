use std::collections::HashMap;

#[derive(Debug, Clone, thiserror::Error)]
pub enum UnitError {
    #[error("unsupported unit conversion")]
    Unsupported,
    #[error("unknown unit")]
    Unknown,
}

/// Unit definition: (base_unit, factor_to_base)
/// Length base: meters (m)
/// Time base: seconds (s)
/// Mass base: kilograms (kg)
/// Volume base: liters (L)
/// Pressure base: Pascals (Pa)
/// Temperature base: Kelvin (K)
/// Amount base: moles (mol)
fn unit_definitions() -> HashMap<&'static str, (&'static str, f64)> {
    let mut m = HashMap::new();

    // Length (base: m)
    m.insert("m", ("m", 1.0));
    m.insert("km", ("m", 1000.0));
    m.insert("cm", ("m", 0.01));
    m.insert("mm", ("m", 0.001));

    // Time (base: s)
    m.insert("s", ("s", 1.0));
    m.insert("min", ("s", 60.0));
    m.insert("h", ("s", 3600.0));

    // Mass (base: kg)
    m.insert("kg", ("kg", 1.0));
    m.insert("g", ("kg", 0.001));

    // Volume (base: L)
    m.insert("L", ("L", 1.0));
    m.insert("mL", ("L", 0.001));

    // Amount (base: mol)
    m.insert("mol", ("mol", 1.0));

    // Pressure (base: Pa)
    m.insert("Pa", ("Pa", 1.0));
    m.insert("bar", ("Pa", 100000.0));
    m.insert("atm_unit", ("Pa", 101325.0));

    // Temperature (special handling)
    m.insert("K", ("K", 1.0));
    m.insert("C", ("K", 1.0));

    m
}

pub fn is_valid_unit(name: &str) -> bool {
    unit_definitions().contains_key(name)
}

pub fn convert(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    if from == to {
        return Ok(value);
    }

    let defs = unit_definitions();
    let (from_base, from_factor) = defs.get(from).ok_or(UnitError::Unknown)?;
    let (to_base, to_factor) = defs.get(to).ok_or(UnitError::Unknown)?;

    if from_base != to_base {
        return Err(UnitError::Unsupported);
    }

    // Special case for temperature
    if *from_base == "K" {
        return convert_temperature(value, from, to);
    }

    let in_base = value * from_factor;
    Ok(in_base / to_factor)
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    let kelvin = match from {
        "K" => value,
        "C" => value + 273.15,
        _ => return Err(UnitError::Unknown),
    };

    match to {
        "K" => Ok(kelvin),
        "C" => Ok(kelvin - 273.15),
        _ => Err(UnitError::Unknown),
    }
}

pub fn get_unit_categories() -> Vec<(String, Vec<String>)> {
    let defs = unit_definitions();
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for (name, (base, _)) in defs.iter() {
        categories
            .entry(base.to_string())
            .or_default()
            .push(name.to_string());
    }

    let mut result: Vec<_> = categories.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}
