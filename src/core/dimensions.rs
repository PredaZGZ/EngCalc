/// Dimensional analysis system for engcalc.
/// Based on SI base dimensions: Length, Time, Mass, Temperature, Current, Amount
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dimensions {
    pub length: i8,      // L - meters
    pub time: i8,        // T - seconds
    pub mass: i8,        // M - kilograms
    pub temperature: i8, // Θ - kelvin
    pub current: i8,     // I - amperes
    pub amount: i8,      // N - moles
}

impl Dimensions {
    pub const NONE: Self = Self {
        length: 0,
        time: 0,
        mass: 0,
        temperature: 0,
        current: 0,
        amount: 0,
    };

    pub const LENGTH: Self = Self {
        length: 1,
        time: 0,
        mass: 0,
        temperature: 0,
        current: 0,
        amount: 0,
    };

    pub const TIME: Self = Self {
        length: 0,
        time: 1,
        mass: 0,
        temperature: 0,
        current: 0,
        amount: 0,
    };

    pub const MASS: Self = Self {
        length: 0,
        time: 0,
        mass: 1,
        temperature: 0,
        current: 0,
        amount: 0,
    };

    pub const TEMPERATURE: Self = Self {
        length: 0,
        time: 0,
        mass: 0,
        temperature: 1,
        current: 0,
        amount: 0,
    };

    pub const CURRENT: Self = Self {
        length: 0,
        time: 0,
        mass: 0,
        temperature: 0,
        current: 1,
        amount: 0,
    };

    pub const AMOUNT: Self = Self {
        length: 0,
        time: 0,
        mass: 0,
        temperature: 0,
        current: 0,
        amount: 1,
    };

    /// Multiply dimensions (for multiplication of values)
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            length: self.length + other.length,
            time: self.time + other.time,
            mass: self.mass + other.mass,
            temperature: self.temperature + other.temperature,
            current: self.current + other.current,
            amount: self.amount + other.amount,
        }
    }

    /// Divide dimensions (for division of values)
    pub fn div(&self, other: &Self) -> Self {
        Self {
            length: self.length - other.length,
            time: self.time - other.time,
            mass: self.mass - other.mass,
            temperature: self.temperature - other.temperature,
            current: self.current - other.current,
            amount: self.amount - other.amount,
        }
    }

    /// Power of dimensions
    pub fn pow(&self, exp: i8) -> Self {
        Self {
            length: self.length * exp,
            time: self.time * exp,
            mass: self.mass * exp,
            temperature: self.temperature * exp,
            current: self.current * exp,
            amount: self.amount * exp,
        }
    }

    /// Check if dimensions are compatible (for addition/subtraction)
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.length == other.length
            && self.time == other.time
            && self.mass == other.mass
            && self.temperature == other.temperature
            && self.current == other.current
            && self.amount == other.amount
    }

    /// Check if dimensionless
    pub fn is_dimensionless(&self) -> bool {
        self.length == 0
            && self.time == 0
            && self.mass == 0
            && self.temperature == 0
            && self.current == 0
            && self.amount == 0
    }

    /// Get a human-readable representation
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        if self.length != 0 {
            parts.push(format_dim("m", self.length));
        }
        if self.time != 0 {
            parts.push(format_dim("s", self.time));
        }
        if self.mass != 0 {
            parts.push(format_dim("kg", self.mass));
        }
        if self.temperature != 0 {
            parts.push(format_dim("K", self.temperature));
        }
        if self.current != 0 {
            parts.push(format_dim("A", self.current));
        }
        if self.amount != 0 {
            parts.push(format_dim("mol", self.amount));
        }
        if parts.is_empty() {
            "dimensionless".to_string()
        } else {
            parts.join("·")
        }
    }
}

fn format_dim(symbol: &str, power: i8) -> String {
    match power {
        1 => symbol.to_string(),
        -1 => format!("{symbol}⁻¹"),
        2 => format!("{symbol}²"),
        -2 => format!("{symbol}⁻²"),
        3 => format!("{symbol}³"),
        -3 => format!("{symbol}⁻³"),
        _ => format!("{symbol}^{power}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_dimensions() {
        let velocity = Dimensions::LENGTH.div(&Dimensions::TIME);
        assert_eq!(velocity.length, 1);
        assert_eq!(velocity.time, -1);
        assert!(velocity.is_compatible(&Dimensions {
            length: 1,
            time: -1,
            mass: 0,
            temperature: 0,
            current: 0,
            amount: 0,
        }));
    }

    #[test]
    fn test_force_dimensions() {
        // F = ma → kg·m/s²
        let force = Dimensions::MASS
            .mul(&Dimensions::LENGTH)
            .div(&Dimensions::TIME.pow(2));
        assert_eq!(force.mass, 1);
        assert_eq!(force.length, 1);
        assert_eq!(force.time, -2);
    }
}
