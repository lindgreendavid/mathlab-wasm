use std::{env, fs, process};

use mathlab_wasm::{solve, Method, Options, SolveResult, Status};
use serde::Serialize;

#[derive(Serialize)]
struct Scenario {
    id: &'static str,
    expectation: &'static str,
    passed: bool,
    result: SolveResult,
}

#[derive(Clone, Copy)]
struct ScenarioSpec {
    id: &'static str,
    expectation: &'static str,
    method: Method,
    function_id: &'static str,
    first: f64,
    second: f64,
    expected_status: Status,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    product_version: &'static str,
    protocol: &'static str,
    arithmetic: &'static str,
    x_tolerance: f64,
    f_tolerance: f64,
    max_iterations: usize,
    all_expectations_met: bool,
    scenarios: Vec<Scenario>,
}

fn scenario(spec: ScenarioSpec, options: Options) -> Scenario {
    let result = solve(
        spec.method,
        spec.function_id,
        spec.first,
        spec.second,
        options,
    );
    Scenario {
        id: spec.id,
        expectation: spec.expectation,
        passed: result.status == spec.expected_status,
        result,
    }
}

fn main() {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "reports/v0.1-root-finding.json".to_owned());
    let options = Options::default();
    let specs = [
        ScenarioSpec {
            id: "cubic-bisection",
            expectation: "converged",
            method: Method::Bisection,
            function_id: "cubic",
            first: 1.0,
            second: 2.0,
            expected_status: Status::Converged,
        },
        ScenarioSpec {
            id: "cubic-newton",
            expectation: "converged",
            method: Method::Newton,
            function_id: "cubic",
            first: 1.5,
            second: 0.0,
            expected_status: Status::Converged,
        },
        ScenarioSpec {
            id: "cosine-secant",
            expectation: "converged",
            method: Method::Secant,
            function_id: "cosine",
            first: 0.0,
            second: 1.0,
            expected_status: Status::Converged,
        },
        ScenarioSpec {
            id: "repeated-bisection",
            expectation: "invalid-bracket",
            method: Method::Bisection,
            function_id: "repeated",
            first: 0.0,
            second: 2.0,
            expected_status: Status::InvalidBracket,
        },
        ScenarioSpec {
            id: "repeated-newton",
            expectation: "converged",
            method: Method::Newton,
            function_id: "repeated",
            first: 2.0,
            second: 0.0,
            expected_status: Status::Converged,
        },
        ScenarioSpec {
            id: "newton-cycle",
            expectation: "cycle-detected",
            method: Method::Newton,
            function_id: "newton-cycle",
            first: 0.0,
            second: 0.0,
            expected_status: Status::CycleDetected,
        },
        ScenarioSpec {
            id: "secant-collapse",
            expectation: "collapsed-secant",
            method: Method::Secant,
            function_id: "repeated",
            first: 0.0,
            second: 2.0,
            expected_status: Status::CollapsedSecant,
        },
    ];
    let scenarios = specs
        .into_iter()
        .map(|spec| scenario(spec, options))
        .collect::<Vec<_>>();
    let all_expectations_met = scenarios.iter().all(|case| case.passed);
    let report = Report {
        schema_version: "1.0.0",
        product_version: env!("CARGO_PKG_VERSION"),
        protocol: "docs/protocol.md",
        arithmetic: "IEEE-754 binary64 via Rust f64",
        x_tolerance: options.x_tolerance,
        f_tolerance: options.f_tolerance,
        max_iterations: options.max_iterations,
        all_expectations_met,
        scenarios,
    };
    let encoded = serde_json::to_string_pretty(&report).expect("serialize report") + "\n";
    fs::write(&output, encoded).expect("write report");
    if !all_expectations_met {
        process::exit(1);
    }
}
