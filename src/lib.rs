//! Deterministic one-dimensional root-finding traces for Mathlab WASM.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const DENOMINATOR_FLOOR: f64 = 64.0 * f64::EPSILON;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    Bisection,
    Newton,
    Secant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Converged,
    InvalidBracket,
    ZeroDerivative,
    CollapsedSecant,
    NonFinite,
    CycleDetected,
    MaxIterations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub iteration: usize,
    pub x: f64,
    pub fx: f64,
    pub step_size: Option<f64>,
    pub bracket_width: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub method: Method,
    pub function_id: String,
    pub status: Status,
    pub root: Option<f64>,
    pub residual: Option<f64>,
    pub iterations: usize,
    pub function_evaluations: usize,
    pub trace: Vec<Step>,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub x_tolerance: f64,
    pub f_tolerance: f64,
    pub max_iterations: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            x_tolerance: 1e-10,
            f_tolerance: 1e-10,
            max_iterations: 80,
        }
    }
}

fn function(function_id: &str, x: f64) -> Option<(f64, f64)> {
    match function_id {
        "cubic" => Some((x * x * x - x - 2.0, 3.0 * x * x - 1.0)),
        "cosine" => Some((x.cos() - x, -x.sin() - 1.0)),
        "repeated" => Some(((x - 1.0).powi(2), 2.0 * (x - 1.0))),
        "newton-cycle" => Some((x * x * x - 2.0 * x + 2.0, 3.0 * x * x - 2.0)),
        "flat" => Some((x.powi(3), 3.0 * x * x)),
        _ => None,
    }
}

fn valid_options(options: Options) -> Options {
    Options {
        x_tolerance: options.x_tolerance.clamp(f64::EPSILON, 1.0),
        f_tolerance: options.f_tolerance.clamp(f64::EPSILON, 1.0),
        max_iterations: options.max_iterations.clamp(1, 500),
    }
}

fn finish(
    method: Method,
    function_id: &str,
    status: Status,
    trace: Vec<Step>,
    evaluations: usize,
    message: &str,
) -> SolveResult {
    let last = trace.last();
    SolveResult {
        method,
        function_id: function_id.to_owned(),
        status,
        root: last.map(|step| step.x).filter(|value| value.is_finite()),
        residual: last
            .map(|step| step.fx.abs())
            .filter(|value| value.is_finite()),
        iterations: last.map_or(0, |step| step.iteration),
        function_evaluations: evaluations,
        trace,
        message: message.to_owned(),
    }
}

pub fn bisection(
    function_id: &str,
    mut left: f64,
    mut right: f64,
    options: Options,
) -> SolveResult {
    let method = Method::Bisection;
    let options = valid_options(options);
    let Some((mut f_left, _)) = function(function_id, left) else {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            0,
            "Unknown function.",
        );
    };
    let Some((f_right, _)) = function(function_id, right) else {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            0,
            "Unknown function.",
        );
    };
    let mut evaluations = 2;
    if !left.is_finite() || !right.is_finite() || !f_left.is_finite() || !f_right.is_finite() {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            evaluations,
            "Inputs must be finite.",
        );
    }
    if left >= right || f_left.signum() == f_right.signum() {
        return finish(
            method,
            function_id,
            Status::InvalidBracket,
            vec![],
            evaluations,
            "The endpoints must be ordered and have opposite signs.",
        );
    }

    let mut trace = Vec::new();
    for iteration in 1..=options.max_iterations {
        let midpoint = left + (right - left) / 2.0;
        let (f_midpoint, _) = function(function_id, midpoint).expect("validated function id");
        evaluations += 1;
        let width = (right - left).abs();
        trace.push(Step {
            iteration,
            x: midpoint,
            fx: f_midpoint,
            step_size: Some(width / 2.0),
            bracket_width: Some(width),
        });
        if !f_midpoint.is_finite() {
            return finish(
                method,
                function_id,
                Status::NonFinite,
                trace,
                evaluations,
                "A non-finite function value stopped the iteration.",
            );
        }
        if f_midpoint.abs() <= options.f_tolerance || width / 2.0 <= options.x_tolerance {
            return finish(
                method,
                function_id,
                Status::Converged,
                trace,
                evaluations,
                "The residual or certified half-width met the frozen tolerance.",
            );
        }
        if f_left.signum() != f_midpoint.signum() {
            right = midpoint;
        } else {
            left = midpoint;
            f_left = f_midpoint;
        }
    }
    finish(
        method,
        function_id,
        Status::MaxIterations,
        trace,
        evaluations,
        "The iteration budget was exhausted.",
    )
}

pub fn newton(function_id: &str, mut x: f64, options: Options) -> SolveResult {
    let method = Method::Newton;
    let options = valid_options(options);
    let mut trace = Vec::new();
    let mut evaluations = 0;
    let mut prior: Vec<f64> = Vec::new();

    for iteration in 0..=options.max_iterations {
        let Some((fx, derivative)) = function(function_id, x) else {
            return finish(
                method,
                function_id,
                Status::NonFinite,
                trace,
                evaluations,
                "Unknown function.",
            );
        };
        evaluations += 2;
        let step_size = trace.last().map(|step: &Step| (x - step.x).abs());
        trace.push(Step {
            iteration,
            x,
            fx,
            step_size,
            bracket_width: None,
        });
        if !x.is_finite() || !fx.is_finite() || !derivative.is_finite() {
            return finish(
                method,
                function_id,
                Status::NonFinite,
                trace,
                evaluations,
                "A non-finite iterate stopped the method.",
            );
        }
        if fx.abs() <= options.f_tolerance
            && step_size.is_none_or(|step| step <= options.x_tolerance)
        {
            return finish(
                method,
                function_id,
                Status::Converged,
                trace,
                evaluations,
                "The residual and step met the frozen tolerances.",
            );
        }
        if iteration == options.max_iterations {
            break;
        }
        if derivative.abs() <= DENOMINATOR_FLOOR * (1.0 + fx.abs()) {
            return finish(
                method,
                function_id,
                Status::ZeroDerivative,
                trace,
                evaluations,
                "The derivative was too small for a defensible Newton step.",
            );
        }
        let next = x - fx / derivative;
        if prior
            .iter()
            .rev()
            .take(8)
            .any(|old| (next - old).abs() <= options.x_tolerance)
        {
            trace.push(Step {
                iteration: iteration + 1,
                x: next,
                fx: function(function_id, next).map_or(f64::NAN, |pair| pair.0),
                step_size: Some((next - x).abs()),
                bracket_width: None,
            });
            evaluations += 1;
            return finish(
                method,
                function_id,
                Status::CycleDetected,
                trace,
                evaluations,
                "A repeated iterate exposed a numerical cycle.",
            );
        }
        prior.push(x);
        x = next;
    }
    finish(
        method,
        function_id,
        Status::MaxIterations,
        trace,
        evaluations,
        "The iteration budget was exhausted.",
    )
}

pub fn secant(
    function_id: &str,
    mut previous: f64,
    mut current: f64,
    options: Options,
) -> SolveResult {
    let method = Method::Secant;
    let options = valid_options(options);
    let Some((mut f_previous, _)) = function(function_id, previous) else {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            0,
            "Unknown function.",
        );
    };
    let Some((mut f_current, _)) = function(function_id, current) else {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            0,
            "Unknown function.",
        );
    };
    let mut evaluations = 2;
    let mut trace = vec![
        Step {
            iteration: 0,
            x: previous,
            fx: f_previous,
            step_size: None,
            bracket_width: None,
        },
        Step {
            iteration: 1,
            x: current,
            fx: f_current,
            step_size: Some((current - previous).abs()),
            bracket_width: None,
        },
    ];
    if [previous, current, f_previous, f_current]
        .iter()
        .any(|value| !value.is_finite())
    {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            trace,
            evaluations,
            "Inputs must be finite.",
        );
    }

    for iteration in 2..=options.max_iterations {
        if f_current.abs() <= options.f_tolerance
            && (current - previous).abs() <= options.x_tolerance
        {
            return finish(
                method,
                function_id,
                Status::Converged,
                trace,
                evaluations,
                "The residual and step met the frozen tolerances.",
            );
        }
        let denominator = f_current - f_previous;
        if denominator.abs() <= DENOMINATOR_FLOOR * (1.0 + f_current.abs().max(f_previous.abs())) {
            return finish(
                method,
                function_id,
                Status::CollapsedSecant,
                trace,
                evaluations,
                "The secant slope denominator collapsed.",
            );
        }
        let next = current - f_current * (current - previous) / denominator;
        let Some((f_next, _)) = function(function_id, next) else {
            unreachable!()
        };
        evaluations += 1;
        trace.push(Step {
            iteration,
            x: next,
            fx: f_next,
            step_size: Some((next - current).abs()),
            bracket_width: None,
        });
        if !next.is_finite() || !f_next.is_finite() {
            return finish(
                method,
                function_id,
                Status::NonFinite,
                trace,
                evaluations,
                "A non-finite iterate stopped the method.",
            );
        }
        previous = current;
        f_previous = f_current;
        current = next;
        f_current = f_next;
    }
    finish(
        method,
        function_id,
        Status::MaxIterations,
        trace,
        evaluations,
        "The iteration budget was exhausted.",
    )
}

pub fn solve(
    method: Method,
    function_id: &str,
    first: f64,
    second: f64,
    options: Options,
) -> SolveResult {
    match method {
        Method::Bisection => bisection(function_id, first, second, options),
        Method::Newton => newton(function_id, first, options),
        Method::Secant => secant(function_id, first, second, options),
    }
}

#[wasm_bindgen]
pub fn solve_json(
    method: &str,
    function_id: &str,
    first: f64,
    second: f64,
    tolerance: f64,
    max_iterations: usize,
) -> Result<String, JsValue> {
    let method = match method {
        "bisection" => Method::Bisection,
        "newton" => Method::Newton,
        "secant" => Method::Secant,
        _ => return Err(JsValue::from_str("Unknown method.")),
    };
    let options = Options {
        x_tolerance: tolerance,
        f_tolerance: tolerance,
        max_iterations,
    };
    serde_json::to_string(&solve(method, function_id, first, second, options))
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
