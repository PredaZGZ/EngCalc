use crate::core::ast::*;
use crate::core::env::Environment;
use crate::core::functions;
use crate::core::units::{self, CompoundUnit};
use crate::core::value::Value;

#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    #[error("division by zero")]
    DivisionByZero,
    #[error("unknown function '{name}'")]
    UnknownFunction { name: String },
    #[error("unknown variable '{name}'")]
    UnknownVariable { name: String },
    #[error("invalid argument for {func}: {reason}")]
    InvalidArgument { func: String, reason: String },
    #[error("cannot convert '{from}' to '{to}': incompatible dimensions")]
    IncompatibleUnits { from: String, to: String },
    #[error("expected unit for conversion, got bare number")]
    NoUnitToConvert,
    #[error("dimensional mismatch: cannot {op} '{left}' and '{right}'")]
    DimensionalMismatch {
        op: String,
        left: String,
        right: String,
    },
    #[error("invalid number of arguments for {name}: expected {expected}, got {got}")]
    ArgCount {
        name: String,
        expected: usize,
        got: usize,
    },
    #[error("incomplete expression")]
    Incomplete,
    #[error("unit not recognized: '{unit}'")]
    UnknownUnit { unit: String },
}

pub fn evaluate(expr: &Expr, env: &Environment) -> Result<Value, EvalError> {
    match expr {
        Expr::Number(n) => Ok(Value::new(*n)),

        Expr::Identifier(name) => {
            if let Some(val) = env.get(name) {
                Ok(val)
            } else if let Some(val) = crate::core::constants::get(name) {
                Ok(val)
            } else {
                Err(EvalError::UnknownVariable { name: name.clone() })
            }
        }

        Expr::BinaryOp { op, left, right } => {
            let l = evaluate(left, env)?;
            let r = evaluate(right, env)?;
            apply_binary(op, l, r)
        }

        Expr::UnaryOp { op, operand } => {
            let val = evaluate(operand, env)?;
            match op {
                UnaryOperator::Neg => {
                    let mut result = val;
                    result.number = -result.number;
                    Ok(result)
                }
            }
        }

        Expr::FunctionCall { name, args } => {
            let values: Result<Vec<Value>, EvalError> =
                args.iter().map(|a| evaluate(a, env)).collect();
            let values = values?;

            // Functions operate on dimensionless numbers, so we strip units
            let nums: Vec<f64> = values.iter().map(|v| v.number).collect();

            let result = functions::call(name, &nums).map_err(|e| match e {
                functions::FuncError::Unknown => EvalError::UnknownFunction { name: name.clone() },
                functions::FuncError::ArgCount { expected, got } => EvalError::ArgCount {
                    name: name.clone(),
                    expected,
                    got,
                },
                functions::FuncError::InvalidArg(reason) => EvalError::InvalidArgument {
                    func: name.clone(),
                    reason,
                },
            })?;

            Ok(Value::new(result))
        }

        Expr::Assignment { name: _, value } => {
            let val = evaluate(value, env)?;
            Ok(val.clone())
        }

        Expr::UnitConvert { value, target_unit } => {
            let val = evaluate(value, env)?;
            if let Some(ref src_unit) = val.unit {
                let src_str = src_unit.to_string();
                let result =
                    units::convert(val.number, &src_str, target_unit).map_err(|e| match e {
                        units::UnitError::DimensionalMismatch(_, _)
                        | units::UnitError::Incompatible { .. } => EvalError::IncompatibleUnits {
                            from: src_str.clone(),
                            to: target_unit.clone(),
                        },
                        _ => EvalError::UnknownUnit {
                            unit: src_str.clone(),
                        },
                    })?;

                let mut new_unit = CompoundUnit::new();
                // Parse target_unit as compound
                if let Ok(compound) = units::parse_compound_unit(target_unit) {
                    Ok(Value::with_unit(result, compound))
                } else {
                    new_unit.add(target_unit, 1);
                    Ok(Value::with_unit(result, new_unit))
                }
            } else {
                Err(EvalError::NoUnitToConvert)
            }
        }

        Expr::UnitValue { value, unit } => {
            let val = evaluate(value, env)?;
            // Try to parse as compound unit
            if let Ok(compound) = units::parse_compound_unit(unit) {
                Ok(Value::with_unit(val.number, compound))
            } else if units::is_valid_unit(unit) {
                let mut compound = CompoundUnit::new();
                compound.add(unit, 1);
                Ok(Value::with_unit(val.number, compound))
            } else {
                Err(EvalError::UnknownUnit { unit: unit.clone() })
            }
        }
    }
}

fn apply_binary(op: &BinaryOperator, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinaryOperator::Add => {
            // Check dimensional compatibility
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "add".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            // If both have units, they must be the same (or convert)
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    // Try to convert r to l's units
                    let r_converted = units::convert(r.number, &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number + r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    // One has unit, one doesn't - just add numbers, keep unit
                    Ok(Value::with_unit(l.number + r.number, u.clone()))
                }
                (None, None) => Ok(Value::new(l.number + r.number)),
            }
        }
        BinaryOperator::Sub => {
            // Same logic as addition
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "subtract".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    let r_converted = units::convert(r.number, &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number - r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    Ok(Value::with_unit(l.number - r.number, u.clone()))
                }
                (None, None) => Ok(Value::new(l.number - r.number)),
            }
        }
        BinaryOperator::Mul => {
            let result_num = l.number * r.number;
            match (l.unit, r.unit) {
                (Some(mut u1), Some(u2)) => {
                    // Multiply units: combine all parts
                    for part in u2.parts {
                        u1.add(&part.name, part.power);
                    }
                    // Simplify if possible
                    if let Some(simplified) = units::simplify_unit(&u1) {
                        let mut new_unit = CompoundUnit::new();
                        new_unit.add(&simplified, 1);
                        Ok(Value::with_unit(result_num, new_unit))
                    } else {
                        Ok(Value::with_unit(result_num, u1))
                    }
                }
                (Some(u), None) | (None, Some(u)) => Ok(Value::with_unit(result_num, u)),
                (None, None) => Ok(Value::new(result_num)),
            }
        }
        BinaryOperator::Div => {
            if r.number == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            let result_num = l.number / r.number;
            match (l.unit, r.unit) {
                (Some(u1), Some(mut u2)) => {
                    // Divide: l units divided by r units = l * r^-1
                    for part in &mut u2.parts {
                        part.power = -part.power;
                    }
                    let mut result_unit = u1;
                    for part in u2.parts {
                        result_unit.add(&part.name, part.power);
                    }
                    // Simplify if possible
                    if let Some(simplified) = units::simplify_unit(&result_unit) {
                        let mut new_unit = CompoundUnit::new();
                        new_unit.add(&simplified, 1);
                        Ok(Value::with_unit(result_num, new_unit))
                    } else {
                        Ok(Value::with_unit(result_num, result_unit))
                    }
                }
                (Some(u), None) => Ok(Value::with_unit(result_num, u)),
                (None, Some(mut u)) => {
                    // Dimensionless / unit = unit^-1
                    for part in &mut u.parts {
                        part.power = -part.power;
                    }
                    Ok(Value::with_unit(result_num, u))
                }
                (None, None) => Ok(Value::new(result_num)),
            }
        }
        BinaryOperator::Mod => {
            if r.number == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            // Modulo requires same units
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "modulo".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    let r_converted = units::convert(r.number, &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number % r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    Ok(Value::with_unit(l.number % r.number, u.clone()))
                }
                (None, None) => Ok(Value::new(l.number % r.number)),
            }
        }
        BinaryOperator::Pow => {
            // Power must be dimensionless
            if r.unit.is_some() {
                return Err(EvalError::InvalidArgument {
                    func: "power".to_string(),
                    reason: "exponent must be dimensionless".to_string(),
                });
            }
            let exp = r.number;
            let result_num = if l.number >= 0.0 || exp.fract() == 0.0 {
                l.number.powf(exp)
            } else {
                l.number.powf(exp) // Will produce NaN for invalid cases
            };
            match l.unit {
                Some(u) => {
                    // Unit^power: raise unit to power
                    let mut result_unit = CompoundUnit::new();
                    for part in u.parts {
                        result_unit.add(&part.name, part.power * exp as i8);
                    }
                    Ok(Value::with_unit(result_num, result_unit))
                }
                None => Ok(Value::new(result_num)),
            }
        }
    }
}
