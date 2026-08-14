use mathlab_wasm::{bisection, newton, safeguarded, secant, Options, Status, StepKind};

const ROOT_CUBIC: f64 = 1.521_379_706_804_567_6;

#[test]
fn bisection_converges_inside_its_certified_bound() {
    let options = Options::default();
    let result = bisection("cubic", 1.0, 2.0, options);
    let bound = ((2.0_f64 - 1.0) / options.x_tolerance).log2().ceil() as usize;
    assert_eq!(result.status, Status::Converged);
    assert!(result.iterations <= bound);
    assert!((result.root.unwrap() - ROOT_CUBIC).abs() < 1e-9);
}

#[test]
fn bisection_rejects_an_unbracketed_even_multiplicity_root() {
    let result = bisection("repeated", 0.0, 2.0, Options::default());
    assert_eq!(result.status, Status::InvalidBracket);
}

#[test]
fn newton_is_fast_near_a_simple_root() {
    let result = newton("cubic", 1.5, Options::default());
    assert_eq!(result.status, Status::Converged);
    assert!(result.iterations <= 5);
    assert!((result.root.unwrap() - ROOT_CUBIC).abs() < 1e-9);
}

#[test]
fn newton_exposes_the_prespecified_two_cycle() {
    let result = newton("newton-cycle", 0.0, Options::default());
    assert_eq!(result.status, Status::CycleDetected);
    assert_eq!(result.trace[0].x, 0.0);
    assert_eq!(result.trace[1].x, 1.0);
    assert_eq!(result.trace[2].x, 0.0);
}

#[test]
fn newton_stops_at_zero_derivative_without_claiming_convergence() {
    let result = newton(
        "flat",
        0.0,
        Options {
            f_tolerance: f64::EPSILON,
            ..Options::default()
        },
    );
    assert_eq!(result.status, Status::Converged);

    let result = newton("newton-cycle", (2.0_f64 / 3.0).sqrt(), Options::default());
    assert_eq!(result.status, Status::ZeroDerivative);
}

#[test]
fn secant_converges_without_a_derivative() {
    let result = secant("cosine", 0.0, 1.0, Options::default());
    assert_eq!(result.status, Status::Converged);
    assert!((result.root.unwrap() - 0.739_085_133_215_160_7).abs() < 1e-9);
}

#[test]
fn secant_detects_a_collapsed_slope() {
    let result = secant("repeated", 0.0, 2.0, Options::default());
    assert_eq!(result.status, Status::CollapsedSecant);
}

#[test]
fn iteration_budget_is_enforced() {
    let result = bisection(
        "cubic",
        1.0,
        2.0,
        Options {
            max_iterations: 1,
            ..Options::default()
        },
    );
    assert_eq!(result.status, Status::MaxIterations);
}

fn assert_brackets_are_certificates(result: &mathlab_wasm::SolveResult, root: f64) {
    let mut previous_width = f64::INFINITY;
    for step in &result.trace {
        let left = step.bracket_left.expect("safeguarded trace left endpoint");
        let right = step
            .bracket_right
            .expect("safeguarded trace right endpoint");
        let width = step.bracket_width.expect("safeguarded trace width");
        assert!(left <= root && root <= right);
        assert!((width - (right - left)).abs() <= 16.0 * f64::EPSILON * (1.0 + width));
        assert!(width <= previous_width + 16.0 * f64::EPSILON * (1.0 + previous_width));
        previous_width = width;
    }
}

#[test]
fn safeguarded_solver_preserves_brackets_for_standard_cases() {
    let cases = [
        ("cubic", 1.0, 2.0, ROOT_CUBIC),
        ("cosine", 0.0, 1.0, 0.739_085_133_215_160_7),
    ];
    for (function_id, left, right, root) in cases {
        let result = safeguarded(function_id, left, right, Options::default());
        assert_eq!(result.status, Status::Converged);
        assert!((result.root.unwrap() - root).abs() < 1e-9);
        assert_brackets_are_certificates(&result, root);
    }
}

#[test]
fn safeguarded_solver_exposes_interpolation_and_fallback() {
    let result = safeguarded("skewed", 0.0, 2.0, Options::default());
    assert_eq!(result.status, Status::Converged);
    assert!((result.root.unwrap() - 1.0).abs() < 1e-9);
    assert!(result.trace.iter().any(|step| matches!(
        step.step_kind,
        Some(StepKind::Secant | StepKind::InverseQuadratic)
    )));
    assert!(result
        .trace
        .iter()
        .any(|step| step.step_kind == Some(StepKind::Bisection)));
    assert_brackets_are_certificates(&result, 1.0);
}

#[test]
fn safeguarded_solver_accepts_an_endpoint_root() {
    let result = safeguarded("flat", 0.0, 1.0, Options::default());
    assert_eq!(result.status, Status::Converged);
    assert_eq!(result.root, Some(0.0));
    assert_eq!(result.iterations, 0);
    assert_eq!(result.function_evaluations, 2);
}

#[test]
fn safeguarded_solver_rejects_a_non_sign_changing_bracket() {
    let result = safeguarded("repeated", 0.0, 2.0, Options::default());
    assert_eq!(result.status, Status::InvalidBracket);
}
