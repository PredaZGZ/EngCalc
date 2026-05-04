use crate::core::value::Value;
use std::fmt;

/// Abstract Syntax Tree for mathematical expressions.
/// Supports binary/unary operations, function calls, variables, numbers,
/// and unit conversions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
    },
    UnitConvert {
        value: Box<Expr>,
        target_unit: String,
    },
    UnitValue {
        value: Box<Expr>,
        unit: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Neg,
}

impl Expr {
    pub fn as_assignment(&self) -> Option<(&str, &Expr)> {
        match self {
            Expr::Assignment { name, value } => Some((name, value)),
            _ => None,
        }
    }
}

/// Display implementation for pretty-printing AST (useful for debugging)
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::UnaryOp { op, operand } => {
                write!(f, "({}{})", op, operand)
            }
            Expr::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Assignment { name, value } => {
                write!(f, "{} = {}", name, value)
            }
            Expr::UnitConvert { value, target_unit } => {
                write!(f, "{} in {}", value, target_unit)
            }
            Expr::UnitValue { value, unit } => {
                write!(f, "{} {}", value, unit)
            }
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Pow => write!(f, "^"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
        }
    }
}

/// Evaluates an Expr to a Value if all variables resolve.
/// This is a convenience method that delegates to the evaluator module.
impl Expr {
    pub fn eval(
        &self,
        env: &crate::core::env::Environment,
    ) -> Result<Value, crate::core::evaluator::EvalError> {
        crate::core::evaluator::evaluate(self, env)
    }
}
