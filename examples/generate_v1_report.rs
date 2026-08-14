use std::{env, fs, process};

use mathlab_wasm::conditioning::build_v1_report;

fn main() {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "reports/v1.0-conditioning.json".to_owned());
    let report = build_v1_report();
    let passed = report.all_expectations_met;
    let encoded = serde_json::to_string_pretty(&report).expect("serialize report") + "\n";
    fs::write(output, encoded).expect("write report");
    if !passed {
        process::exit(1);
    }
}
