#[derive(Debug, Clone, thiserror::Error)]
pub enum FuncError {
    #[error("unknown function")]
    Unknown,
    #[error("invalid argument count: expected {expected}, got {got}")]
    ArgCount { expected: usize, got: usize },
    #[error("invalid argument: {0}")]
    InvalidArg(String),
}

pub fn call(name: &str, args: &[f64]) -> Result<f64, FuncError> {
    match name {
        "sin" => unary(args, f64::sin),
        "cos" => unary(args, f64::cos),
        "tan" => unary(args, f64::tan),
        "asin" => unary(args, f64::asin),
        "acos" => unary(args, f64::acos),
        "atan" => unary(args, f64::atan),
        "sqrt" => unary(args, |x| {
            if x < 0.0 {
                f64::NAN
            } else {
                x.sqrt()
            }
        }),
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
    )
}

pub fn function_names() -> Vec<&'static str> {
    vec![
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "sqrt",
        "ln",
        "log",
        "log10",
        "exp",
        "abs",
        "floor",
        "ceil",
        "round",
        "min",
        "max",
        "pow",
    ]
}
