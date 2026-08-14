//! Deterministic one-dimensional root-finding traces for Mathlab WASM.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const DENOMINATOR_FLOOR: f64 = 64.0 * f64::EPSILON;

fn safe_midpoint(left: f64, right: f64) -> f64 {
    left / 2.0 + right / 2.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    Bisection,
    Newton,
    Secant,
    Safeguarded,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepKind {
    Bisection,
    Secant,
    InverseQuadratic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub iteration: usize,
    pub x: f64,
    pub fx: f64,
    pub step_size: Option<f64>,
    pub bracket_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bracket_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_kind: Option<StepKind>,
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
        "skewed" => Some((x.powi(10) - 1.0, 10.0 * x.powi(9))),
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
            bracket_left: None,
            bracket_right: None,
            step_kind: None,
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
            bracket_left: None,
            bracket_right: None,
            step_kind: None,
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
                bracket_left: None,
                bracket_right: None,
                step_kind: None,
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
            bracket_left: None,
            bracket_right: None,
            step_kind: None,
        },
        Step {
            iteration: 1,
            x: current,
            fx: f_current,
            step_size: Some((current - previous).abs()),
            bracket_width: None,
            bracket_left: None,
            bracket_right: None,
            step_kind: None,
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
            bracket_left: None,
            bracket_right: None,
            step_kind: None,
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

pub fn safeguarded(function_id: &str, mut a: f64, mut b: f64, options: Options) -> SolveResult {
    let method = Method::Safeguarded;
    let options = valid_options(options);
    let Some((mut fa, _)) = function(function_id, a) else {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            0,
            "Unknown function.",
        );
    };
    let Some((mut fb, _)) = function(function_id, b) else {
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
    if [a, b, fa, fb].iter().any(|value| !value.is_finite()) {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            evaluations,
            "Inputs must be finite.",
        );
    }
    if a >= b {
        return finish(
            method,
            function_id,
            Status::InvalidBracket,
            vec![],
            evaluations,
            "The endpoints must be ordered.",
        );
    }
    if !(b - a).is_finite() {
        return finish(
            method,
            function_id,
            Status::NonFinite,
            vec![],
            evaluations,
            "The bracket width must be finite.",
        );
    }
    if fa == 0.0 || fb == 0.0 {
        let (root, f_root) = if fa == 0.0 { (a, fa) } else { (b, fb) };
        let trace = vec![Step {
            iteration: 0,
            x: root,
            fx: f_root,
            step_size: None,
            bracket_width: Some((b - a).abs()),
            bracket_left: Some(a),
            bracket_right: Some(b),
            step_kind: None,
        }];
        return finish(
            method,
            function_id,
            Status::Converged,
            trace,
            evaluations,
            "An endpoint is an exact root in binary64 arithmetic.",
        );
    }
    if fa.signum() == fb.signum() {
        return finish(
            method,
            function_id,
            Status::InvalidBracket,
            vec![],
            evaluations,
            "The endpoint function values must have opposite signs.",
        );
    }

    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = c;
    let mut used_bisection = true;
    let mut trace = Vec::new();

    for iteration in 1..=options.max_iterations {
        let bracket_left = a.min(b);
        let bracket_right = a.max(b);
        let bracket_width = bracket_right - bracket_left;
        if bracket_width / 2.0 <= options.x_tolerance {
            let midpoint = safe_midpoint(bracket_left, bracket_right);
            let (f_midpoint, _) = function(function_id, midpoint).expect("validated function id");
            evaluations += 1;
            trace.push(Step {
                iteration,
                x: midpoint,
                fx: f_midpoint,
                step_size: Some((midpoint - b).abs()),
                bracket_width: Some(bracket_width),
                bracket_left: Some(bracket_left),
                bracket_right: Some(bracket_right),
                step_kind: Some(StepKind::Bisection),
            });
            let (status, message) = if f_midpoint.is_finite() {
                (
                    Status::Converged,
                    "The certified bracket half-width met the frozen tolerance.",
                )
            } else {
                (
                    Status::NonFinite,
                    "A non-finite midpoint stopped the method.",
                )
            };
            return finish(method, function_id, status, trace, evaluations, message);
        }

        let local_scale = fa.abs().max(fb.abs()).max(fc.abs());
        let denominator_floor = DENOMINATOR_FLOOR * (1.0 + local_scale);
        let can_use_iqi = (fa - fb).abs() > denominator_floor
            && (fa - fc).abs() > denominator_floor
            && (fb - fc).abs() > denominator_floor;
        let (mut candidate, mut step_kind) = if can_use_iqi {
            let first = a * fb * fc / ((fa - fb) * (fa - fc));
            let second = b * fa * fc / ((fb - fa) * (fb - fc));
            let third = c * fa * fb / ((fc - fa) * (fc - fb));
            (first + second + third, StepKind::InverseQuadratic)
        } else if (fb - fa).abs() > denominator_floor {
            (b - fb * (b - a) / (fb - fa), StepKind::Secant)
        } else {
            (safe_midpoint(a, b), StepKind::Bisection)
        };

        let interpolation_bound = (3.0 * a + b) / 4.0;
        let lower_bound = interpolation_bound.min(b);
        let upper_bound = interpolation_bound.max(b);
        let outside_safe_region =
            !candidate.is_finite() || candidate <= lower_bound || candidate >= upper_bound;
        let insufficient_progress = if used_bisection {
            (candidate - b).abs() >= (b - c).abs() / 2.0 || (b - c).abs() < options.x_tolerance
        } else {
            (candidate - b).abs() >= (c - d).abs() / 2.0 || (c - d).abs() < options.x_tolerance
        };
        if step_kind == StepKind::Bisection || outside_safe_region || insufficient_progress {
            candidate = safe_midpoint(a, b);
            step_kind = StepKind::Bisection;
            used_bisection = true;
        } else {
            used_bisection = false;
        }

        let previous_best = b;
        let (f_candidate, _) = function(function_id, candidate).expect("validated function id");
        evaluations += 1;
        if !candidate.is_finite() || !f_candidate.is_finite() {
            trace.push(Step {
                iteration,
                x: candidate,
                fx: f_candidate,
                step_size: Some((candidate - previous_best).abs()),
                bracket_width: Some(bracket_width),
                bracket_left: Some(bracket_left),
                bracket_right: Some(bracket_right),
                step_kind: Some(step_kind),
            });
            return finish(
                method,
                function_id,
                Status::NonFinite,
                trace,
                evaluations,
                "A non-finite interpolation result stopped the method.",
            );
        }

        d = c;
        c = b;
        fc = fb;
        if fa.signum() != f_candidate.signum() {
            b = candidate;
            fb = f_candidate;
        } else {
            a = candidate;
            fa = f_candidate;
        }
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }

        let next_left = a.min(b);
        let next_right = a.max(b);
        trace.push(Step {
            iteration,
            x: candidate,
            fx: f_candidate,
            step_size: Some((candidate - previous_best).abs()),
            bracket_width: Some(next_right - next_left),
            bracket_left: Some(next_left),
            bracket_right: Some(next_right),
            step_kind: Some(step_kind),
        });
        if f_candidate.abs() <= options.f_tolerance {
            return finish(
                method,
                function_id,
                Status::Converged,
                trace,
                evaluations,
                "The residual met the frozen tolerance while the bracket was preserved.",
            );
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
        Method::Safeguarded => safeguarded(function_id, first, second, options),
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
        "safeguarded" => Method::Safeguarded,
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
