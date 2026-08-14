//! Prespecified residual and conditioning diagnostics for the v1.0 study.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_COMMIT: &str = "97b21a2";
const CUBIC_ROOT: f64 = 1.521_379_706_804_567_6;
const LINEAR_CANDIDATE: f64 = 1.000_001;
const LINEAR_ROOT: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditioningChecks {
    pub finite_required_values: bool,
    pub classification_matches: bool,
    pub linear_estimate_matches_forward_error: Option<bool>,
    pub cubic_relative_estimate_error_below_1e_7: Option<bool>,
    pub repeated_root_contract_met: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditioningCase {
    pub id: String,
    pub equation: String,
    pub candidate: f64,
    pub reference_root: f64,
    pub residual: f64,
    pub forward_error: f64,
    pub derivative_magnitude_at_root: f64,
    pub absolute_condition_number: Option<f64>,
    pub first_order_error_estimate: Option<f64>,
    pub classification: String,
    pub interpretation: String,
    pub checks: ConditioningChecks,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalChecks {
    pub linear_forward_errors_identical: bool,
    pub residual_order_flat_unit_steep: bool,
    pub condition_order_flat_unit_steep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditioningReport {
    pub schema_version: String,
    pub product_version: String,
    pub protocol: String,
    pub protocol_commit: String,
    pub arithmetic: String,
    pub perturbation_model: String,
    pub interpretation: String,
    pub global_checks: GlobalChecks,
    pub all_expectations_met: bool,
    pub cases: Vec<ConditioningCase>,
}

#[derive(Clone, Copy)]
struct CaseSpec {
    id: &'static str,
    equation: &'static str,
    candidate: f64,
    root: f64,
    scale: f64,
    kind: CaseKind,
    classification: &'static str,
    interpretation: &'static str,
}

#[derive(Clone, Copy)]
enum CaseKind {
    Linear,
    Cubic,
    Repeated,
}

fn evaluate(spec: CaseSpec) -> ConditioningCase {
    let (value, derivative_at_root) = match spec.kind {
        CaseKind::Linear => (spec.scale * (spec.candidate - spec.root), spec.scale),
        CaseKind::Cubic => (
            spec.candidate.powi(3) - spec.candidate - 2.0,
            3.0 * spec.root.powi(2) - 1.0,
        ),
        CaseKind::Repeated => ((spec.candidate - spec.root).powi(2), 0.0),
    };
    let residual = value.abs();
    let forward_error = (spec.candidate - spec.root).abs();
    let derivative_magnitude_at_root = derivative_at_root.abs();
    let absolute_condition_number =
        (derivative_magnitude_at_root > 0.0).then_some(1.0 / derivative_magnitude_at_root);
    let first_order_error_estimate =
        absolute_condition_number.map(|condition| residual * condition);
    let linear_estimate_matches_forward_error = matches!(spec.kind, CaseKind::Linear).then(|| {
        (first_order_error_estimate.expect("linear estimate") - forward_error).abs()
            <= 32.0 * f64::EPSILON * (1.0 + forward_error)
    });
    let cubic_relative_estimate_error_below_1e_7 =
        matches!(spec.kind, CaseKind::Cubic).then(|| {
            ((first_order_error_estimate.expect("cubic estimate") - forward_error).abs()
                / forward_error)
                <= 1e-7
        });
    let repeated_root_contract_met = matches!(spec.kind, CaseKind::Repeated).then(|| {
        derivative_magnitude_at_root == 0.0
            && absolute_condition_number.is_none()
            && first_order_error_estimate.is_none()
            && residual < 1e-9
            && forward_error > 1e-6
    });
    let finite_required_values = [
        spec.candidate,
        spec.root,
        residual,
        forward_error,
        derivative_magnitude_at_root,
    ]
    .into_iter()
    .all(f64::is_finite)
        && absolute_condition_number.is_none_or(f64::is_finite)
        && first_order_error_estimate.is_none_or(f64::is_finite);
    let classification_matches = match spec.kind {
        CaseKind::Linear if spec.scale < 1.0 => {
            spec.classification == "small-residual-high-sensitivity"
        }
        CaseKind::Linear if spec.scale > 1.0 => {
            spec.classification == "large-residual-low-sensitivity"
        }
        CaseKind::Linear => spec.classification == "residual-tracks-error",
        CaseKind::Cubic => spec.classification == "local-linearization-valid",
        CaseKind::Repeated => spec.classification == "simple-root-diagnostic-unavailable",
    };
    let checks = ConditioningChecks {
        finite_required_values,
        classification_matches,
        linear_estimate_matches_forward_error,
        cubic_relative_estimate_error_below_1e_7,
        repeated_root_contract_met,
    };
    let passed = checks.finite_required_values
        && checks.classification_matches
        && checks.linear_estimate_matches_forward_error.unwrap_or(true)
        && checks
            .cubic_relative_estimate_error_below_1e_7
            .unwrap_or(true)
        && checks.repeated_root_contract_met.unwrap_or(true);
    ConditioningCase {
        id: spec.id.to_owned(),
        equation: spec.equation.to_owned(),
        candidate: spec.candidate,
        reference_root: spec.root,
        residual,
        forward_error,
        derivative_magnitude_at_root,
        absolute_condition_number,
        first_order_error_estimate,
        classification: spec.classification.to_owned(),
        interpretation: spec.interpretation.to_owned(),
        checks,
        passed,
    }
}

/// Builds the complete prespecified v1.0 report without external input.
pub fn build_v1_report() -> ConditioningReport {
    let specifications = [
        CaseSpec {
            id: "flat-scaled-linear",
            equation: "1e-8 * (x - 1)",
            candidate: LINEAR_CANDIDATE,
            root: LINEAR_ROOT,
            scale: 1e-8,
            kind: CaseKind::Linear,
            classification: "small-residual-high-sensitivity",
            interpretation: "The equation scale makes the residual tiny although the candidate is no closer to the root.",
        },
        CaseSpec {
            id: "unit-linear",
            equation: "x - 1",
            candidate: LINEAR_CANDIDATE,
            root: LINEAR_ROOT,
            scale: 1.0,
            kind: CaseKind::Linear,
            classification: "residual-tracks-error",
            interpretation: "For this unit-scaled linear equation, residual and forward error coincide.",
        },
        CaseSpec {
            id: "steep-scaled-linear",
            equation: "1e8 * (x - 1)",
            candidate: LINEAR_CANDIDATE,
            root: LINEAR_ROOT,
            scale: 1e8,
            kind: CaseKind::Linear,
            classification: "large-residual-low-sensitivity",
            interpretation: "The equation scale makes the residual large although the candidate has the same root error.",
        },
        CaseSpec {
            id: "simple-cubic-local",
            equation: "x^3 - x - 2",
            candidate: CUBIC_ROOT + 1e-8,
            root: CUBIC_ROOT,
            scale: 1.0,
            kind: CaseKind::Cubic,
            classification: "local-linearization-valid",
            interpretation: "Near this simple root, the derivative-based first-order estimate tracks the forward error.",
        },
        CaseSpec {
            id: "repeated-root",
            equation: "(x - 1)^2",
            candidate: 1.000_01,
            root: 1.0,
            scale: 1.0,
            kind: CaseKind::Repeated,
            classification: "simple-root-diagnostic-unavailable",
            interpretation: "The derivative vanishes at the multiple root, so the simple-root linearization must not be reported as a finite bound.",
        },
    ];
    let cases = specifications.into_iter().map(evaluate).collect::<Vec<_>>();
    let linear = &cases[..3];
    let global_checks = GlobalChecks {
        linear_forward_errors_identical: linear
            .windows(2)
            .all(|pair| pair[0].forward_error == pair[1].forward_error),
        residual_order_flat_unit_steep: linear[0].residual < linear[1].residual
            && linear[1].residual < linear[2].residual,
        condition_order_flat_unit_steep: linear[0].absolute_condition_number
            > linear[1].absolute_condition_number
            && linear[1].absolute_condition_number > linear[2].absolute_condition_number,
    };
    let all_expectations_met = cases.iter().all(|case| case.passed)
        && global_checks.linear_forward_errors_identical
        && global_checks.residual_order_flat_unit_steep
        && global_checks.condition_order_flat_unit_steep;
    ConditioningReport {
        schema_version: "1.0.0".to_owned(),
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        protocol: "docs/protocol-v1.0.md".to_owned(),
        protocol_commit: PROTOCOL_COMMIT.to_owned(),
        arithmetic: "IEEE-754 binary64 via Rust f64".to_owned(),
        perturbation_model: "absolute additive perturbation to the function value".to_owned(),
        interpretation: "Five prespecified demonstrations; not a prevalence estimate, rigorous enclosure, or universal stopping rule.".to_owned(),
        global_checks,
        all_expectations_met,
        cases,
    }
}
