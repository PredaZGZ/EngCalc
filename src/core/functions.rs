#[derive(Debug, Clone, thiserror::Error)]
pub enum FuncError {
    #[error("unknown function")]
    Unknown,
    #[error("invalid argument count: expected {expected}, got {got}")]
    ArgCount { expected: usize, got: usize },
    #[error("invalid argument: {0}")]
    InvalidArg(String),
}

pub struct FunctionInfo {
    pub name: &'static str,
    pub params: &'static str,
    pub description: &'static str,
}

pub fn list_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "sin",
            params: "x",
            description: "Sine (radians)",
        },
        FunctionInfo {
            name: "cos",
            params: "x",
            description: "Cosine (radians)",
        },
        FunctionInfo {
            name: "tan",
            params: "x",
            description: "Tangent (radians)",
        },
        FunctionInfo {
            name: "asin",
            params: "x",
            description: "Arc sine",
        },
        FunctionInfo {
            name: "acos",
            params: "x",
            description: "Arc cosine",
        },
        FunctionInfo {
            name: "atan",
            params: "x",
            description: "Arc tangent",
        },
        FunctionInfo {
            name: "sqrt",
            params: "x",
            description: "Square root",
        },
        FunctionInfo {
            name: "ln",
            params: "x",
            description: "Natural logarithm",
        },
        FunctionInfo {
            name: "log",
            params: "x",
            description: "Base-10 logarithm",
        },
        FunctionInfo {
            name: "log10",
            params: "x",
            description: "Base-10 logarithm",
        },
        FunctionInfo {
            name: "exp",
            params: "x",
            description: "Exponential e^x",
        },
        FunctionInfo {
            name: "abs",
            params: "x",
            description: "Absolute value",
        },
        FunctionInfo {
            name: "floor",
            params: "x",
            description: "Round down",
        },
        FunctionInfo {
            name: "ceil",
            params: "x",
            description: "Round up",
        },
        FunctionInfo {
            name: "round",
            params: "x",
            description: "Round to nearest",
        },
        FunctionInfo {
            name: "min",
            params: "a, b",
            description: "Minimum of two",
        },
        FunctionInfo {
            name: "max",
            params: "a, b",
            description: "Maximum of two",
        },
        FunctionInfo {
            name: "pow",
            params: "base, exp",
            description: "Power base^exp",
        },
        // Integration functions
        FunctionInfo {
            name: "trapz",
            params: "f, a, b, n",
            description: "Integral by trapezoidal rule (n intervals)",
        },
        FunctionInfo {
            name: "simpson",
            params: "f, a, b, n",
            description: "Integral by Simpson's rule (n intervals, n even)",
        },
        FunctionInfo {
            name: "rkf45",
            params: "f, a, b, [tol], [max_steps]",
            description: "Integral by RKF45 adaptive quadrature",
        },
    ]
}

pub fn call(name: &str, args: &[f64]) -> Result<f64, FuncError> {
    match name {
        "sin" => unary(args, f64::sin),
        "cos" => unary(args, f64::cos),
        "tan" => unary(args, f64::tan),
        "asin" => unary(args, f64::asin),
        "acos" => unary(args, f64::acos),
        "atan" => unary(args, f64::atan),
        "sqrt" => unary(args, |x| if x < 0.0 { f64::NAN } else { x.sqrt() }),
        "ln" => unary(args, f64::ln),
        "log" => unary(args, f64::log10),
        "log10" => unary(args, f64::log10),
        "exp" => unary(args, f64::exp),
        "abs" => unary(args, f64::abs),
        "floor" => unary(args, f64::floor),
        "ceil" => unary(args, f64::ceil),
        "round" => unary(args, f64::round),
        "min" => binary(args, f64::min),
        "max" => binary(args, f64::max),
        "pow" => binary(args, f64::powf),
        _ => Err(FuncError::Unknown),
    }
}

fn unary(args: &[f64], f: fn(f64) -> f64) -> Result<f64, FuncError> {
    if args.len() != 1 {
        return Err(FuncError::ArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    Ok(f(args[0]))
}

fn binary(args: &[f64], f: fn(f64, f64) -> f64) -> Result<f64, FuncError> {
    if args.len() != 2 {
        return Err(FuncError::ArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    Ok(f(args[0], args[1]))
}

pub fn is_function(name: &str) -> bool {
    matches!(
        name,
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sqrt"
            | "ln"
            | "log"
            | "log10"
            | "exp"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
            | "pow"
            | "trapz"
            | "simpson"
            | "rkf45"
    )
}

pub fn function_names() -> Vec<&'static str> {
    vec![
        "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "ln", "log", "log10", "exp", "abs",
        "floor", "ceil", "round", "min", "max", "pow", "trapz", "simpson", "rkf45",
    ]
}
