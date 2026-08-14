use std::{env, fs, process};

use mathlab_wasm::{safeguarded, Options, SolveResult, Status, StepKind};
use serde::Serialize;

const PROTOCOL_COMMIT: &str = "e4c6f222c22f163b909503d05ead800394757f26";
const WIDTH_ALLOWANCE: f64 = 16.0 * f64::EPSILON;

#[derive(Serialize)]
struct Checks {
    status_matches: bool,
    reference_error_below_1e_9: Option<bool>,
    brackets_contain_reference: Option<bool>,
    bracket_width_non_increasing: Option<bool>,
    interpolation_observed: Option<bool>,
    bisection_observed: Option<bool>,
    endpoint_contract_met: Option<bool>,
}

#[derive(Serialize)]
struct Scenario {
    id: &'static str,
    expectation: &'static str,
    reference_root: Option<f64>,
    checks: Checks,
    passed: bool,
    result: SolveResult,
}

#[derive(Clone, Copy)]
struct ScenarioSpec {
    id: &'static str,
    expectation: &'static str,
    function_id: &'static str,
    left: f64,
    right: f64,
    expected_status: Status,
    reference_root: Option<f64>,
    requires_step_mix: bool,
    requires_endpoint_contract: bool,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    product_version: &'static str,
    protocol: &'static str,
    protocol_commit: &'static str,
    arithmetic: &'static str,
    x_tolerance: f64,
    f_tolerance: f64,
    max_iterations: usize,
    interpretation: &'static str,
    all_expectations_met: bool,
    scenarios: Vec<Scenario>,
}

fn bracket_checks(result: &SolveResult, reference: f64) -> (bool, bool) {
    let contains = result.trace.iter().all(|step| {
        step.bracket_left
            .zip(step.bracket_right)
            .is_some_and(|(left, right)| left <= reference && reference <= right)
    });
    let non_increasing = result.trace.windows(2).all(|pair| {
        pair[0]
            .bracket_width
            .zip(pair[1].bracket_width)
            .is_some_and(|(previous, current)| {
                current <= previous + WIDTH_ALLOWANCE * (1.0 + previous)
            })
    });
    (contains, non_increasing)
}

fn evaluate(spec: ScenarioSpec, options: Options) -> Scenario {
    let result = safeguarded(spec.function_id, spec.left, spec.right, options);
    let status_matches = result.status == spec.expected_status;
    let reference_error_below_1e_9 = spec
        .reference_root
        .zip(result.root)
        .map(|(reference, estimate)| (estimate - reference).abs() < 1e-9);
    let (brackets_contain_reference, bracket_width_non_increasing) = match spec.reference_root {
        Some(reference) if !result.trace.is_empty() => {
            let (contains, non_increasing) = bracket_checks(&result, reference);
            (Some(contains), Some(non_increasing))
        }
        _ => (None, None),
    };
    let interpolation_observed = spec.requires_step_mix.then(|| {
        result.trace.iter().any(|step| {
            matches!(
                step.step_kind,
                Some(StepKind::Secant | StepKind::InverseQuadratic)
            )
        })
    });
    let bisection_observed = spec.requires_step_mix.then(|| {
        result
            .trace
            .iter()
            .any(|step| step.step_kind == Some(StepKind::Bisection))
    });
    let endpoint_contract_met = spec.requires_endpoint_contract.then(|| {
        result.root == Some(0.0) && result.iterations == 0 && result.function_evaluations == 2
    });
    let checks = Checks {
        status_matches,
        reference_error_below_1e_9,
        brackets_contain_reference,
        bracket_width_non_increasing,
        interpolation_observed,
        bisection_observed,
        endpoint_contract_met,
    };
    let passed = checks.status_matches
        && checks.reference_error_below_1e_9.unwrap_or(true)
        && checks.brackets_contain_reference.unwrap_or(true)
        && checks.bracket_width_non_increasing.unwrap_or(true)
        && checks.interpolation_observed.unwrap_or(true)
        && checks.bisection_observed.unwrap_or(true)
        && checks.endpoint_contract_met.unwrap_or(true);
    Scenario {
        id: spec.id,
        expectation: spec.expectation,
        reference_root: spec.reference_root,
        checks,
        passed,
        result,
    }
}

fn main() {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "reports/v0.2-safeguarded-root-finding.json".to_owned());
    let options = Options::default();
    let specs = [
        ScenarioSpec {
            id: "cubic-safeguarded",
            expectation: "converged",
            function_id: "cubic",
            left: 1.0,
            right: 2.0,
            expected_status: Status::Converged,
            reference_root: Some(1.521_379_706_804_567_6),
            requires_step_mix: false,
            requires_endpoint_contract: false,
        },
        ScenarioSpec {
            id: "cosine-safeguarded",
            expectation: "converged",
            function_id: "cosine",
            left: 0.0,
            right: 1.0,
            expected_status: Status::Converged,
            reference_root: Some(0.739_085_133_215_160_7),
            requires_step_mix: false,
            requires_endpoint_contract: false,
        },
        ScenarioSpec {
            id: "skewed-safeguarded",
            expectation: "converged-with-interpolation-and-bisection",
            function_id: "skewed",
            left: 0.0,
            right: 2.0,
            expected_status: Status::Converged,
            reference_root: Some(1.0),
            requires_step_mix: true,
            requires_endpoint_contract: false,
        },
        ScenarioSpec {
            id: "endpoint-safeguarded",
            expectation: "exact-endpoint-root",
            function_id: "flat",
            left: 0.0,
            right: 1.0,
            expected_status: Status::Converged,
            reference_root: Some(0.0),
            requires_step_mix: false,
            requires_endpoint_contract: true,
        },
        ScenarioSpec {
            id: "repeated-safeguarded",
            expectation: "invalid-bracket",
            function_id: "repeated",
            left: 0.0,
            right: 2.0,
            expected_status: Status::InvalidBracket,
            reference_root: None,
            requires_step_mix: false,
            requires_endpoint_contract: false,
        },
    ];
    let scenarios = specs
        .into_iter()
        .map(|spec| evaluate(spec, options))
        .collect::<Vec<_>>();
    let all_expectations_met = scenarios.iter().all(|case| case.passed);
    let report = Report {
        schema_version: "1.0.0",
        product_version: env!("CARGO_PKG_VERSION"),
        protocol: "docs/protocol-v0.2.md",
        protocol_commit: PROTOCOL_COMMIT,
        arithmetic: "IEEE-754 binary64 via Rust f64",
        x_tolerance: options.x_tolerance,
        f_tolerance: options.f_tolerance,
        max_iterations: options.max_iterations,
        interpretation: "Selected deterministic demonstrations; not a representative solver benchmark or universal ranking.",
        all_expectations_met,
        scenarios,
    };
    let encoded = serde_json::to_string_pretty(&report).expect("serialize report") + "\n";
    fs::write(&output, encoded).expect("write report");
    if !all_expectations_met {
        process::exit(1);
    }
}
