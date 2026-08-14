#!/usr/bin/env python3
"""Validate the committed v0.1 machine-readable result."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports" / "v0.1-root-finding.json"


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
    print("v0.1 report is structurally valid and all frozen expectations pass")


if __name__ == "__main__":
    main()
