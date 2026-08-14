use mathlab_wasm::conditioning::build_v1_report;

#[test]
fn v1_protocol_expectations_are_met() {
    let report = build_v1_report();
    assert!(report.all_expectations_met);
    assert!(report.global_checks.linear_forward_errors_identical);
    assert!(report.global_checks.residual_order_flat_unit_steep);
    assert!(report.global_checks.condition_order_flat_unit_steep);
}

#[test]
fn equation_scaling_changes_residual_not_forward_error() {
    let report = build_v1_report();
    let linear = &report.cases[..3];
    assert_eq!(linear[0].forward_error, linear[1].forward_error);
    assert_eq!(linear[1].forward_error, linear[2].forward_error);
    assert!(linear[0].residual < linear[1].residual);
    assert!(linear[1].residual < linear[2].residual);
}

#[test]
fn repeated_root_does_not_receive_a_simple_root_bound() {
    let report = build_v1_report();
    let repeated = report
        .cases
        .iter()
        .find(|case| case.id == "repeated-root")
        .expect("repeated-root case");
    assert_eq!(repeated.derivative_magnitude_at_root, 0.0);
    assert_eq!(repeated.absolute_condition_number, None);
    assert_eq!(repeated.first_order_error_estimate, None);
    assert!(repeated.residual < 1e-9);
    assert!(repeated.forward_error > 1e-6);
}

#[test]
fn cubic_first_order_estimate_meets_frozen_tolerance() {
    let report = build_v1_report();
    let cubic = report
        .cases
        .iter()
        .find(|case| case.id == "simple-cubic-local")
        .expect("cubic case");
    let estimate = cubic
        .first_order_error_estimate
        .expect("simple root estimate");
    let relative_error = (estimate - cubic.forward_error).abs() / cubic.forward_error;
    assert!(relative_error <= 1e-7);
}
