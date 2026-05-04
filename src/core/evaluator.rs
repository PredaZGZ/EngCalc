use crate::core::ast::*;
use crate::core::env::Environment;
use crate::core::functions;
use crate::core::units;
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
    #[error("unsupported unit conversion: '{from}' to '{to}'")]
    UnsupportedUnitConversion { from: String, to: String },
    #[error("expected unit for conversion, got bare number")]
    NoUnitToConvert,
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
            apply_binary(op, l.number, r.number)
        }

        Expr::UnaryOp { op, operand } => {
            let val = evaluate(operand, env)?;
            match op {
                UnaryOperator::Neg => Ok(Value::new(-val.number)),
            }
        }

        Expr::FunctionCall { name, args } => {
            let values: Result<Vec<Value>, EvalError> =
                args.iter().map(|a| evaluate(a, env)).collect();
            let values = values?;

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

        Expr::Assignment { name: _name, value } => {
            let val = evaluate(value, env)?;
            // Note: the actual storage happens in app.rs after evaluation succeeds
            Ok(val.clone())
        }

        Expr::UnitConvert { value, target_unit } => {
            let val = evaluate(value, env)?;
            if let Some(ref src_unit) = val.unit {
                let result = units::convert(val.number, src_unit, target_unit).map_err(|e| {
                    if let units::UnitError::Unsupported = e {
                        EvalError::UnsupportedUnitConversion {
                            from: src_unit.clone(),
                            to: target_unit.clone(),
                        }
                    } else {
                        EvalError::UnknownUnit {
                            unit: src_unit.clone(),
                        }
                    }
                })?;
                Ok(Value::new(result))
            } else {
                Err(EvalError::NoUnitToConvert)
            }
        }

        Expr::UnitValue { value, unit } => {
            let val = evaluate(value, env)?;
            if units::is_valid_unit(unit) {
                Ok(Value::with_unit(val.number, unit.clone()))
            } else {
                Err(EvalError::UnknownUnit { unit: unit.clone() })
            }
        }
    }
}

fn apply_binary(op: &BinaryOperator, l: f64, r: f64) -> Result<Value, EvalError> {
    match op {
        BinaryOperator::Add => Ok(Value::new(l + r)),
        BinaryOperator::Sub => Ok(Value::new(l - r)),
        BinaryOperator::Mul => Ok(Value::new(l * r)),
        BinaryOperator::Div => {
            if r == 0.0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::new(l / r))
            }
        }
        BinaryOperator::Mod => {
            if r == 0.0 {
                Err(EvalError::DivisionByZero)
            } else {
                Ok(Value::new(l % r))
            }
        }
        BinaryOperator::Pow => {
            let result = if r >= 0.0 && l >= 0.0 {
                l.powf(r)
            } else if l < 0.0 && r.fract() == 0.0 {
                l.powf(r)
            } else {
                l.powf(r)
            };
            Ok(Value::new(result))
        }
    }
}
