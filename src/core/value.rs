use std::fmt;

/// Represents a computed value in the calculator.
/// Supports f64 numbers with optional unit annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub number: f64,
    pub unit: Option<String>,
}

impl Value {
    pub fn new(number: f64) -> Self {
        Self {
            number,
            unit: None,
        }
    }

    pub fn with_unit(number: f64, unit: String) -> Self {
        Self {
            number,
            unit: Some(unit),
        }
    }

    pub fn is_nan(&self) -> bool {
        self.number.is_nan()
    }

    pub fn is_infinite(&self) -> bool {
        self.number.is_infinite()
    }

    pub fn has_unit(&self) -> bool {
        self.unit.is_some()
    }

    pub fn strip_unit(&self) -> Self {
        Self {
            number: self.number,
            unit: None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nan() {
            write!(f, "NaN")
        } else if self.number.is_infinite() {
            if self.number > 0.0 {
                write!(f, "∞")
            } else {
                write!(f, "-∞")
            }
        } else if let Some(ref unit) = self.unit {
            write!(f, "{} {}", self.number, unit)
        } else {
            write!(f, "{}", self.number)
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::new(n)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Self::new(n as f64)
    }
}
