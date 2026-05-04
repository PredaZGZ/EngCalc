use crate::core::env::Environment;
use crate::core::evaluator::evaluate;
use crate::core::ast::Expr;
use crate::core::value::Value;

/// Numerical integration using the trapezoidal rule
/// integral(f, a, b, n) approximates the definite integral of f from a to b using n trapezoids
pub fn trapezoidal(
    func_body: &Expr,
    param_name: &str,
    a: f64,
    b: f64,
    n: usize,
    env: &Environment,
) -> Result<f64, String> {
    if n == 0 {
        return Err("Number of intervals must be > 0".to_string());
    }
    if a >= b {
        return Err("Lower bound must be less than upper bound".to_string());
    }

    let h = (b - a) / n as f64;
    let mut sum = 0.0;

    // f(a) + f(b)
    let fa = evaluate_at(func_body, param_name, a, env)
        .ok_or("Failed to evaluate function at lower bound")?;
    let fb = evaluate_at(func_body, param_name, b, env)
        .ok_or("Failed to evaluate function at upper bound")?;

    sum += (fa + fb) / 2.0;

    // Sum of f(x_i) for i = 1 to n-1
    for i in 1..n {
        let x = a + i as f64 * h;
        let fx = evaluate_at(func_body, param_name, x, env)
            .ok_or_else(|| format!("Failed to evaluate function at x = {}", x))?;
        sum += fx;
    }

    Ok(sum * h)
}

/// Numerical integration using Simpson's rule (parabolic approximation)
/// simpson(f, a, b, n) approximates the integral using n/2 parabolic segments
/// n must be even
pub fn simpson(
    func_body: &Expr,
    param_name: &str,
    a: f64,
    b: f64,
    n: usize,
    env: &Environment,
) -> Result<f64, String> {
    if n == 0 {
        return Err("Number of intervals must be > 0".to_string());
    }
    if n % 2 != 0 {
        return Err("Number of intervals must be even for Simpson's rule".to_string());
    }
    if a >= b {
        return Err("Lower bound must be less than upper bound".to_string());
    }

    let h = (b - a) / n as f64;
    let mut sum = 0.0;

    // f(a) + f(b)
    let fa = evaluate_at(func_body, param_name, a, env)
        .ok_or("Failed to evaluate function at lower bound")?;
    let fb = evaluate_at(func_body, param_name, b, env)
        .ok_or("Failed to evaluate function at upper bound")?;

    sum += fa + fb;

    // Odd indices: 4 * f(x_i)
    for i in (1..n).step_by(2) {
        let x = a + i as f64 * h;
        let fx = evaluate_at(func_body, param_name, x, env)
            .ok_or_else(|| format!("Failed to evaluate function at x = {}", x))?;
        sum += 4.0 * fx;
    }

    // Even indices: 2 * f(x_i)
    for i in (2..n).step_by(2) {
        let x = a + i as f64 * h;
        let fx = evaluate_at(func_body, param_name, x, env)
            .ok_or_else(|| format!("Failed to evaluate function at x = {}", x))?;
        sum += 2.0 * fx;
    }

    Ok(sum * h / 3.0)
}

/// Numerical integration using 4th-order Runge-Kutta-Fehlberg (RKF45)
/// This is an adaptive method that adjusts step size for desired accuracy
pub fn rkf45(
    func_body: &Expr,
    param_name: &str,
    a: f64,
    b: f64,
    tolerance: f64,
    max_steps: usize,
    env: &Environment,
) -> Result<f64, String> {
    if a >= b {
        return Err("Lower bound must be less than upper bound".to_string());
    }
    if tolerance <= 0.0 {
        return Err("Tolerance must be positive".to_string());
    }

    let mut x = a;
    let mut integral = 0.0;
    let mut h = (b - a) / 100.0; // Initial step size
    let mut steps = 0;

    // RKF45 Butcher tableau coefficients
    const A2: f64 = 1.0 / 4.0;
    const A3: f64 = 3.0 / 8.0;
    const A4: f64 = 12.0 / 13.0;
    const A5: f64 = 1.0;
    const A6: f64 = 1.0 / 2.0;

    const B21: f64 = 1.0 / 4.0;
    const B31: f64 = 3.0 / 32.0;
    const B32: f64 = 9.0 / 32.0;
    const B41: f64 = 1932.0 / 2197.0;
    const B42: f64 = -7200.0 / 2197.0;
    const B43: f64 = 7296.0 / 2197.0;
    const B51: f64 = 439.0 / 216.0;
    const B52: f64 = -8.0;
    const B53: f64 = 3680.0 / 513.0;
    const B54: f64 = -845.0 / 4104.0;
    const B61: f64 = -8.0 / 27.0;
    const B62: f64 = 2.0;
    const B63: f64 = -3544.0 / 2565.0;
    const B64: f64 = 1859.0 / 4104.0;
    const B65: f64 = -11.0 / 40.0;

    const CH1: f64 = 16.0 / 135.0;
    const CH2: f64 = 0.0;
    const CH3: f64 = 6656.0 / 12825.0;
    const CH4: f64 = 28561.0 / 56430.0;
    const CH5: f64 = -9.0 / 50.0;
    const CH6: f64 = 2.0 / 55.0;

    const CT1: f64 = 1.0 / 360.0;
    const CT2: f64 = 0.0;
    const CT3: f64 = -128.0 / 4275.0;
    const CT4: f64 = -2197.0 / 75240.0;
    const CT5: f64 = 1.0 / 50.0;
    const CT6: f64 = 2.0 / 55.0;

    while x < b && steps < max_steps {
        // Adjust step size to not overshoot b
        if x + h > b {
            h = b - x;
        }

        let k1 = h * evaluate_at(func_body, param_name, x, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x))?;
        let k2 = h * evaluate_at(func_body, param_name, x + A2 * h, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x + A2 * h))?;
        let k3 = h * evaluate_at(func_body, param_name, x + A3 * h, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x + A3 * h))?;
        let k4 = h * evaluate_at(func_body, param_name, x + A4 * h, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x + A4 * h))?;
        let k5 = h * evaluate_at(func_body, param_name, x + A5 * h, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x + A5 * h))?;
        let k6 = h * evaluate_at(func_body, param_name, x + A6 * h, env)
            .ok_or_else(|| format!("Failed to evaluate at x = {}", x + A6 * h))?;

        // 5th order solution
        let y5 = CH1 * k1 + CH2 * k2 + CH3 * k3 + CH4 * k4 + CH5 * k5 + CH6 * k6;

        // Error estimate (4th order - 5th order)
        let error = (CT1 * k1 + CT2 * k2 + CT3 * k3 + CT4 * k4 + CT5 * k5 + CT6 * k6).abs();

        // Adjust step size based on error
        if error > tolerance {
            // Reduce step size
            h *= 0.5;
            if h < (b - a) / 1e6 {
                return Err("Step size too small - function may be ill-conditioned".to_string());
            }
            continue;
        }

        // Accept this step
        integral += y5;
        x += h;
        steps += 1;

        // Increase step size if error is small
        if error < tolerance / 10.0 {
            h *= 2.0;
        }
    }

    if steps >= max_steps {
        return Err("Maximum number of steps exceeded".to_string());
    }

    Ok(integral)
}

fn evaluate_at(
    func_body: &Expr,
    param_name: &str,
    x: f64,
    env: &Environment,
) -> Option<f64> {
    let mut local_env = Environment::new();
    
    // Copy all variables and functions from parent environment
    local_env.copy_from(env);

    local_env.set(param_name.to_string(), Value::new(x));

    match evaluate(func_body, &local_env) {
        Ok(val) => Some(val.number()),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parser;

    fn parse(expr: &str) -> Expr {
        parser::parse(expr).unwrap()
    }

    #[test]
    fn test_trapezoidal_x_squared() {
        let env = Environment::new();
        let ast = parse("x^2");
        // Integral of x^2 from 0 to 1 = 1/3
        let result = trapezoidal(&ast, "x", 0.0, 1.0, 100, &env).unwrap();
        assert!((result - 1.0 / 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_simpson_x_squared() {
        let env = Environment::new();
        let ast = parse("x^2");
        // Integral of x^2 from 0 to 1 = 1/3 (Simpson is exact for quadratics)
        let result = simpson(&ast, "x", 0.0, 1.0, 100, &env).unwrap();
        assert!((result - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_trapezoidal_sin() {
        let env = Environment::new();
        let ast = parse("sin(x)");
        // Integral of sin(x) from 0 to pi = 2
        let result = trapezoidal(&ast, "x", 0.0, std::f64::consts::PI, 1000, &env).unwrap();
        assert!((result - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_rkf45_x_squared() {
        let env = Environment::new();
        let ast = parse("x^2");
        // Integral of x^2 from 0 to 1 = 1/3
        let result = rkf45(&ast, "x", 0.0, 1.0, 1e-8, 10000, &env).unwrap();
        assert!((result - 1.0 / 3.0).abs() < 1e-6);
    }
}
