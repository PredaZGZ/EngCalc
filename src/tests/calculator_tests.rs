use crate::core::env::Environment;
use crate::core::formatter;
use crate::core::parser;

fn eval(input: &str) -> Result<String, String> {
    let ast = parser::parse(input).map_err(|e| e.to_string())?;
    let mut env = Environment::new();

    // Pre-populate with constants
    for (name, _desc, val) in crate::core::constants::list() {
        env.set(name, crate::core::value::Value::new(val));
    }

    match ast.eval(&env) {
        Ok(value) => Ok(formatter::format_value(&value)),
        Err(e) => Err(e.to_string()),
    }
}

fn eval_with_env(input: &str) -> Result<(String, Environment), String> {
    let ast = parser::parse(input).map_err(|e| e.to_string())?;
    let mut env = Environment::new();

    for (name, _desc, val) in crate::core::constants::list() {
        env.set(name, crate::core::value::Value::new(val));
    }

    match ast.eval(&env) {
        Ok(value) => {
            if let Some((name, val_expr)) = ast.as_assignment() {
                let val = val_expr.eval(&env).map_err(|e| e.to_string())?;
                env.set(name.to_string(), val.clone());
            }
            Ok((formatter::format_value(&value), env))
        }
        Err(e) => Err(e.to_string()),
    }
}

fn assert_result(input: &str, expected: &str) {
    let result = eval(input).expect(&format!("Expected success for '{}', but got error", input));
    assert_eq!(
        result, expected,
        "For '{}': expected '{}', got '{}'",
        input, expected, result
    );
}

fn assert_approx(input: &str, expected: f64, tolerance: f64) {
    let result = eval(input).expect(&format!("Expected success for '{}', but got error", input));
    let num: f64 = result
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse result '{}' as f64", result));
    assert!(
        (num - expected).abs() < tolerance,
        "For '{}': expected ~{}, got {}",
        input,
        expected,
        result
    );
}

fn assert_error(input: &str) {
    let result = eval(input);
    assert!(result.is_err(), "Expected error for '{}', but got: {:?}", input, result);
}

#[test]
fn test_basic_addition() {
    assert_result("2 + 2", "4");
}

#[test]
fn test_basic_subtraction() {
    assert_result("10 - 3", "7");
}

#[test]
fn test_basic_multiplication() {
    assert_result("3 * 4", "12");
}

#[test]
fn test_basic_division() {
    assert_result("15 / 3", "5");
}

#[test]
fn test_precedence_mul_before_add() {
    assert_result("2 + 3 * 4", "14");
}

#[test]
fn test_parentheses_override_precedence() {
    assert_result("(2 + 3) * 4", "20");
}

#[test]
fn test_power() {
    assert_result("2^3", "8");
}

#[test]
fn test_power_right_associative() {
    assert_result("2^3^2", "512");
}

#[test]
fn test_unary_minus_power() {
    assert_result("-2^2", "-4");
}

#[test]
fn test_unary_minus_parentheses() {
    assert_result("(-2)^2", "4");
}

#[test]
fn test_modulo() {
    assert_result("10 % 3", "1");
}

#[test]
fn test_implicit_multiplication_number_variable() {
    let (result, env) = eval_with_env("x = 5").unwrap();
    assert_eq!(result, "5");

    let ast = parser::parse("2x").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "10");
}

#[test]
fn test_implicit_multiplication_number_parentheses() {
    assert_result("2(3 + 4)", "14");
}

#[test]
fn test_implicit_multiplication_parentheses_parentheses() {
    assert_result("(2 + 3)(4 + 5)", "45");
}

#[test]
fn test_sin_pi_over_2() {
    assert_approx("sin(pi / 2)", 1.0, 1e-10);
}

#[test]
fn test_cos_zero() {
    assert_approx("cos(0)", 1.0, 1e-10);
}

#[test]
fn test_sqrt() {
    assert_result("sqrt(16)", "4");
}

#[test]
fn test_ln_e() {
    assert_approx("ln(e)", 1.0, 1e-10);
}

#[test]
fn test_log10() {
    assert_result("log10(1000)", "3");
}

#[test]
fn test_exp() {
    assert_approx("exp(0)", 1.0, 1e-10);
}

#[test]
fn test_abs() {
    assert_result("abs(-5)", "5");
}

#[test]
fn test_floor() {
    assert_result("floor(3.7)", "3");
}

#[test]
fn test_ceil() {
    assert_result("ceil(3.2)", "4");
}

#[test]
fn test_round() {
    assert_result("round(3.6)", "4");
}

#[test]
fn test_min_max() {
    assert_result("min(3, 7)", "3");
    assert_result("max(3, 7)", "7");
}

#[test]
fn test_pow_function() {
    assert_result("pow(2, 3)", "8");
}

#[test]
fn test_constant_pi() {
    assert_approx("pi", 3.141592653589793, 1e-10);
}

#[test]
fn test_constant_e() {
    assert_approx("e", 2.718281828459045, 1e-10);
}

#[test]
fn test_constant_tau() {
    assert_approx("tau", 6.283185307179586, 1e-10);
}

#[test]
fn test_variable_assignment_and_use() {
    let (_, env) = eval_with_env("x = 5").unwrap();
    let ast = parser::parse("2x + 3").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "13");
}

#[test]
fn test_variable_chain() {
    let (_, env) = eval_with_env("r = 3").unwrap();
    let ast = parser::parse("pi * r^2").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_approx(&formatter::format_value(&val), 28.2743338823, 1e-6);
}

#[test]
fn test_unit_km_to_m() {
    let ast = parser::parse("10 km in m").unwrap();
    let mut env = Environment::new();
    for (name, _desc, val) in crate::core::constants::list() {
        env.set(name, crate::core::value::Value::new(val));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "10000");
}

#[test]
fn test_unit_h_to_s() {
    let ast = parser::parse("1 h in s").unwrap();
    let mut env = Environment::new();
    for (name, _desc, val) in crate::core::constants::list() {
        env.set(name, crate::core::value::Value::new(val));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "3600");
}

#[test]
fn test_unit_bar_to_pa() {
    let ast = parser::parse("5 bar in Pa").unwrap();
    let mut env = Environment::new();
    for (name, _desc, val) in crate::core::constants::list() {
        env.set(name, crate::core::value::Value::new(val));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "500000");
}

#[test]
fn test_division_by_zero() {
    assert_error("1 / 0");
}

#[test]
fn test_unknown_function() {
    assert_error("foo(3)");
}

#[test]
fn test_unknown_variable() {
    assert_error("x");
}

#[test]
fn test_unmatched_parentheses() {
    assert_error("(2 + 3");
}

#[test]
fn test_formatter_integer_display() {
    let val = crate::core::value::Value::new(12.0);
    assert_eq!(formatter::format_value(&val), "12");
}

#[test]
fn test_formatter_decimal_display() {
    let val = crate::core::value::Value::new(3.14159);
    let result = formatter::format_value(&val);
    assert!(result.starts_with("3.14"));
}

#[test]
fn test_formatter_nan() {
    let val = crate::core::value::Value::new(f64::NAN);
    assert_eq!(formatter::format_value(&val), "NaN");
}

#[test]
fn test_formatter_infinity() {
    let val = crate::core::value::Value::new(f64::INFINITY);
    assert_eq!(formatter::format_value(&val), "∞");

    let val = crate::core::value::Value::new(f64::NEG_INFINITY);
    assert_eq!(formatter::format_value(&val), "-∞");
}

#[test]
fn test_engineering_constants() {
    assert_approx("R", 8.314462618, 1e-6);
    assert_approx("g", 9.80665, 1e-6);
    assert_approx("atm", 101325.0, 1e-3);
}

#[test]
fn test_complex_expression() {
    assert_approx("sin(pi / 4) * sqrt(2)", 1.0, 1e-10);
}

#[test]
fn test_nested_functions() {
    assert_approx("sin(cos(0))", 0.8414709848, 1e-10);
}
