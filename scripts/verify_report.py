#!/usr/bin/env python3
"""Validate the committed versioned machine-readable results."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports" / "v0.1-root-finding.json"
REPORT_V0_2 = ROOT / "reports" / "v0.2-safeguarded-root-finding.json"
REPORT_V1 = ROOT / "reports" / "v1.0-conditioning.json"


def main() -> None:
    payload = json.loads(REPORT.read_text(encoding="utf-8"))
    assert payload["schema_version"] == "1.0.0"
    assert payload["product_version"] == "0.1.0"
    assert payload["all_expectations_met"] is True
    assert len(payload["scenarios"]) == 7
    ids = {case["id"] for case in payload["scenarios"]}
    assert ids == {
        "cubic-bisection",
        "cubic-newton",
        "cosine-secant",
        "repeated-bisection",
        "repeated-newton",
        "newton-cycle",
        "secant-collapse",
    }
    assert all(case["passed"] for case in payload["scenarios"])

    safeguarded = json.loads(REPORT_V0_2.read_text(encoding="utf-8"))
    assert safeguarded["schema_version"] == "1.0.0"
    assert safeguarded["product_version"] == "0.2.0"
    assert safeguarded["protocol"] == "docs/protocol-v0.2.md"
    assert (
        safeguarded["protocol_commit"]
        == "e4c6f222c22f163b909503d05ead800394757f26"
    )
    assert safeguarded["all_expectations_met"] is True
    assert len(safeguarded["scenarios"]) == 5
    assert {case["id"] for case in safeguarded["scenarios"]} == {
        "cubic-safeguarded",
        "cosine-safeguarded",
        "skewed-safeguarded",
        "endpoint-safeguarded",
        "repeated-safeguarded",
    }
    assert all(case["passed"] for case in safeguarded["scenarios"])
    skewed = next(
        case for case in safeguarded["scenarios"] if case["id"] == "skewed-safeguarded"
    )
    kinds = {step.get("step_kind") for step in skewed["result"]["trace"]}
    assert "bisection" in kinds
    assert kinds & {"secant", "inverse-quadratic"}
    conditioning = json.loads(REPORT_V1.read_text(encoding="utf-8"))
    assert conditioning["schema_version"] == "1.0.0"
    assert conditioning["product_version"] == "1.0.0"
    assert conditioning["protocol"] == "docs/protocol-v1.0.md"
    assert conditioning["protocol_commit"] == "97b21a2"
    assert conditioning["all_expectations_met"] is True
    assert len(conditioning["cases"]) == 5
    assert {case["id"] for case in conditioning["cases"]} == {
        "flat-scaled-linear",
        "unit-linear",
        "steep-scaled-linear",
        "simple-cubic-local",
        "repeated-root",
    }
    assert all(case["passed"] for case in conditioning["cases"])
    assert all(conditioning["global_checks"].values())
    repeated = next(case for case in conditioning["cases"] if case["id"] == "repeated-root")
    assert repeated["absolute_condition_number"] is None
    assert repeated["first_order_error_estimate"] is None
    assert (ROOT / "web" / "data" / "v1.0-conditioning.json").read_bytes() == REPORT_V1.read_bytes()
    print("v0.1, v0.2, and v1.0 reports are structurally valid; all frozen expectations pass")


if __name__ == "__main__":
    main()
