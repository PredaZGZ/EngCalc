use crate::core::dimensions::Dimensions;
use std::fmt;

#[derive(Debug, Clone, thiserror::Error)]
pub enum UnitError {
    #[error("incompatible units: cannot convert from '{from}' to '{to}'")]
    Incompatible { from: String, to: String },
    #[error("unknown unit: '{0}'")]
    Unknown(String),
    #[error("dimensional mismatch: {0} vs {1}")]
    DimensionalMismatch(String, String),
}

/// A unit with its conversion factor to SI base and its dimensions
#[derive(Debug, Clone)]
pub struct UnitDef {
    pub name: &'static str,
    pub factor: f64, // factor to convert to SI base unit
    pub dimensions: Dimensions,
    pub category: &'static str,
}

/// Compound unit expression (e.g., "km/h", "m*s^2")
#[derive(Debug, Clone)]
pub struct CompoundUnit {
    pub parts: Vec<UnitPart>,
}

#[derive(Debug, Clone)]
pub struct UnitPart {
    pub name: String,
    pub power: i8, // positive for numerator, negative for denominator
}

impl CompoundUnit {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn add(&mut self, name: &str, power: i8) {
        // Check if already exists and combine
        for part in &mut self.parts {
            if part.name == name {
                part.power += power;
                return;
            }
        }
        self.parts.push(UnitPart {
            name: name.to_string(),
            power,
        });
    }

    /// Calculate total dimensions and conversion factor
    pub fn to_dimensions_and_factor(&self) -> Result<(Dimensions, f64), UnitError> {
        let mut dims = Dimensions::NONE;
        let mut factor = 1.0;

        for part in &self.parts {
            if let Some(def) = get_unit_def(&part.name) {
                // Apply power to dimensions
                let part_dims = if part.power > 0 {
                    def.dimensions.pow(part.power as i8)
                } else {
                    def.dimensions.pow(-part.power)
                };

                if part.power > 0 {
                    dims = dims.mul(&part_dims);
                    factor *= def.factor.powi(part.power as i32);
                } else {
                    dims = dims.div(&part_dims);
                    factor /= def.factor.powi((-part.power) as i32);
                }
            } else {
                return Err(UnitError::Unknown(part.name.clone()));
            }
        }

        Ok((dims, factor))
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty() || self.parts.iter().all(|p| p.power == 0)
    }

    pub fn to_string(&self) -> String {
        let num: Vec<_> = self.parts.iter().filter(|p| p.power > 0).collect();
        let den: Vec<_> = self.parts.iter().filter(|p| p.power < 0).collect();

        let num_str = if num.is_empty() {
            "1".to_string()
        } else {
            num.iter()
                .map(|p| format_unit_power(&p.name, p.power))
                .collect::<Vec<_>>()
                .join("·")
        };

        if den.is_empty() {
            num_str
        } else {
            let den_str = den
                .iter()
                .map(|p| format_unit_power(&p.name, -p.power))
                .collect::<Vec<_>>()
                .join("·");
            format!("{}/{}", num_str, den_str)
        }
    }
}

impl fmt::Display for CompoundUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for CompoundUnit {
    fn eq(&self, other: &Self) -> bool {
        // Compare normalized forms
        let self_str = self.to_string();
        let other_str = other.to_string();
        self_str == other_str
    }
}

fn format_unit_power(name: &str, power: i8) -> String {
    match power {
        1 => name.to_string(),
        2 => format!("{}²", name),
        3 => format!("{}³", name),
        _ => format!("{}^{}", name, power),
    }
}

fn unit_definitions() -> Vec<UnitDef> {
    vec![
        // Length (base: m)
        UnitDef {
            name: "m",
            factor: 1.0,
            dimensions: Dimensions::LENGTH,
            category: "length",
        },
        UnitDef {
            name: "km",
            factor: 1000.0,
            dimensions: Dimensions::LENGTH,
            category: "length",
        },
        UnitDef {
            name: "cm",
            factor: 0.01,
            dimensions: Dimensions::LENGTH,
            category: "length",
        },
        UnitDef {
            name: "mm",
            factor: 0.001,
            dimensions: Dimensions::LENGTH,
            category: "length",
        },
        // Time (base: s)
        UnitDef {
            name: "s",
            factor: 1.0,
            dimensions: Dimensions::TIME,
            category: "time",
        },
        UnitDef {
            name: "min",
            factor: 60.0,
            dimensions: Dimensions::TIME,
            category: "time",
        },
        UnitDef {
            name: "h",
            factor: 3600.0,
            dimensions: Dimensions::TIME,
            category: "time",
        },
        // Mass (base: kg)
        UnitDef {
            name: "kg",
            factor: 1.0,
            dimensions: Dimensions::MASS,
            category: "mass",
        },
        UnitDef {
            name: "g",
            factor: 0.001,
            dimensions: Dimensions::MASS,
            category: "mass",
        },
        // Temperature (base: K)
        UnitDef {
            name: "K",
            factor: 1.0,
            dimensions: Dimensions::TEMPERATURE,
            category: "temperature",
        },
        UnitDef {
            name: "C",
            factor: 1.0,
            dimensions: Dimensions::TEMPERATURE,
            category: "temperature",
        },
        // Amount (base: mol)
        UnitDef {
            name: "mol",
            factor: 1.0,
            dimensions: Dimensions::AMOUNT,
            category: "amount",
        },
        // Current (base: A)
        UnitDef {
            name: "A",
            factor: 1.0,
            dimensions: Dimensions::CURRENT,
            category: "current",
        },
        // Pressure (derived: Pa = N/m² = kg/(m·s²))
        UnitDef {
            name: "Pa",
            factor: 1.0,
            dimensions: Dimensions {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimensions::NONE
            },
            category: "pressure",
        },
        UnitDef {
            name: "bar",
            factor: 100000.0,
            dimensions: Dimensions {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimensions::NONE
            },
            category: "pressure",
        },
        UnitDef {
            name: "atm",
            factor: 101325.0,
            dimensions: Dimensions {
                mass: 1,
                length: -1,
                time: -2,
                ..Dimensions::NONE
            },
            category: "pressure",
        },
        // Force (derived: N = kg·m/s²)
        UnitDef {
            name: "N",
            factor: 1.0,
            dimensions: Dimensions {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimensions::NONE
            },
            category: "force",
        },
        UnitDef {
            name: "kN",
            factor: 1000.0,
            dimensions: Dimensions {
                mass: 1,
                length: 1,
                time: -2,
                ..Dimensions::NONE
            },
            category: "force",
        },
        // Energy (derived: J = N·m = kg·m²/s²)
        UnitDef {
            name: "J",
            factor: 1.0,
            dimensions: Dimensions {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimensions::NONE
            },
            category: "energy",
        },
        UnitDef {
            name: "kJ",
            factor: 1000.0,
            dimensions: Dimensions {
                mass: 1,
                length: 2,
                time: -2,
                ..Dimensions::NONE
            },
            category: "energy",
        },
        // Power (derived: W = J/s = kg·m²/s³)
        UnitDef {
            name: "W",
            factor: 1.0,
            dimensions: Dimensions {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimensions::NONE
            },
            category: "power",
        },
        UnitDef {
            name: "kW",
            factor: 1000.0,
            dimensions: Dimensions {
                mass: 1,
                length: 2,
                time: -3,
                ..Dimensions::NONE
            },
            category: "power",
        },
        // Frequency (derived: Hz = s⁻¹)
        UnitDef {
            name: "Hz",
            factor: 1.0,
            dimensions: Dimensions {
                time: -1,
                ..Dimensions::NONE
            },
            category: "frequency",
        },
        UnitDef {
            name: "kHz",
            factor: 1000.0,
            dimensions: Dimensions {
                time: -1,
                ..Dimensions::NONE
            },
            category: "frequency",
        },
        UnitDef {
            name: "MHz",
            factor: 1_000_000.0,
            dimensions: Dimensions {
                time: -1,
                ..Dimensions::NONE
            },
            category: "frequency",
        },
    ]
}

fn get_unit_def(name: &str) -> Option<UnitDef> {
    unit_definitions().into_iter().find(|u| u.name == name)
}

pub fn is_valid_unit(name: &str) -> bool {
    get_unit_def(name).is_some()
}

pub fn is_simple_unit(name: &str) -> bool {
    // Check if it's a single unit without / or * or ^
    !name.contains('/') && !name.contains('*') && !name.contains('^')
}

/// Parse a compound unit string like "km/h" or "m*s^2"
pub fn parse_compound_unit(input: &str) -> Result<CompoundUnit, UnitError> {
    let mut compound = CompoundUnit::new();
    let mut current = String::new();
    let mut in_denominator = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' => {
                if !current.is_empty() {
                    compound.add(&current, if in_denominator { -1 } else { 1 });
                    current.clear();
                }
                in_denominator = true;
            }
            '*' | '·' => {
                if !current.is_empty() {
                    compound.add(&current, if in_denominator { -1 } else { 1 });
                    current.clear();
                }
            }
            '^' => {
                // Parse exponent
                if !current.is_empty() {
                    let base = current.clone();
                    current.clear();
                    // Read exponent (can be negative)
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch == '-' {
                            chars.next();
                            current.push('-');
                        }
                    }
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            current.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Ok(exp) = current.parse::<i8>() {
                        compound.add(&base, if in_denominator { -exp } else { exp });
                    }
                    current.clear();
                }
            }
            _ if ch.is_alphabetic() || ch == '°' => {
                current.push(ch);
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        compound.add(&current, if in_denominator { -1 } else { 1 });
    }

    if compound.is_empty() {
        return Err(UnitError::Unknown(input.to_string()));
    }

    Ok(compound)
}

/// Convert value from one compound unit to another
pub fn convert(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    let from_unit = parse_compound_unit(from)?;
    let to_unit = parse_compound_unit(to)?;

    let (from_dims, from_factor) = from_unit.to_dimensions_and_factor()?;
    let (to_dims, to_factor) = to_unit.to_dimensions_and_factor()?;

    if !from_dims.is_compatible(&to_dims) {
        return Err(UnitError::DimensionalMismatch(
            from_dims.to_string(),
            to_dims.to_string(),
        ));
    }

    // Convert: value * from_factor / to_factor
    Ok(value * from_factor / to_factor)
}

/// Get all unit categories with their units
pub fn get_unit_categories() -> Vec<(String, Vec<String>)> {
    use std::collections::HashMap;
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for def in unit_definitions() {
        categories
            .entry(def.category.to_string())
            .or_default()
            .push(def.name.to_string());
    }

    let mut result: Vec<_> = categories.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Simplify compound unit to a derived unit if possible
pub fn simplify_unit(compound: &CompoundUnit) -> Option<String> {
    let (dims, _factor) = compound.to_dimensions_and_factor().ok()?;

    // Check against known derived units
    for def in unit_definitions() {
        if def.dimensions.is_compatible(&dims) && def.name.len() <= 3 {
            // Simple derived unit
            return Some(def.name.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_km_per_h() {
        let unit = parse_compound_unit("km/h").unwrap();
        assert_eq!(unit.parts.len(), 2);
        let (dims, factor) = unit.to_dimensions_and_factor().unwrap();
        assert_eq!(dims.length, 1);
        assert_eq!(dims.time, -1);
        // km/h = 1000 m / 3600 s ≈ 0.277... m/s
        assert!((factor - 1000.0 / 3600.0).abs() < 1e-10);
    }

    #[test]
    fn test_convert_kmh_to_ms() {
        let result = convert(36.0, "km/h", "m/s").unwrap();
        assert!((result - 10.0).abs() < 1e-10); // 36 km/h = 10 m/s
    }

    #[test]
    fn test_incompatible_units() {
        let result = convert(10.0, "m", "s");
        assert!(result.is_err());
    }

    #[test]
    fn test_newtons_to_kg_m_s2() {
        // 1 N = 1 kg·m/s²
        let result = convert(1.0, "N", "kg*m/s^2").unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }
}
