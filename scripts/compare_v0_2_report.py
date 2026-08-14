#!/usr/bin/env python3
"""Compare v0.2 reports exactly except for tightly bounded floating-point leaves."""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import Any

FLOAT_ALLOWANCE = 16.0 * sys.float_info.epsilon


def compare(expected: Any, observed: Any, path: str = "$") -> None:
    if isinstance(expected, bool) or isinstance(observed, bool):
        assert expected is observed, f"{path}: boolean differs"
        return
    assert type(expected) is type(observed), f"{path}: JSON type differs"
    if isinstance(expected, float) or isinstance(observed, float):
        tolerance = FLOAT_ALLOWANCE * (1.0 + max(abs(expected), abs(observed)))
        assert math.isfinite(expected) and math.isfinite(observed), f"{path}: non-finite number"
        assert abs(expected - observed) <= tolerance, (
            f"{path}: {expected!r} != {observed!r} within {tolerance:.3e}"
        )
        return
    if isinstance(expected, dict):
        assert isinstance(observed, dict), f"{path}: object type differs"
        assert list(expected) == list(observed), f"{path}: object keys or order differ"
        for key in expected:
            compare(expected[key], observed[key], f"{path}.{key}")
        return
    if isinstance(expected, list):
        assert isinstance(observed, list), f"{path}: list type differs"
        assert len(expected) == len(observed), f"{path}: list length differs"
        for index, (expected_item, observed_item) in enumerate(zip(expected, observed)):
            compare(expected_item, observed_item, f"{path}[{index}]")
        return
    assert expected == observed, f"{path}: {expected!r} != {observed!r}"


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: compare_v0_2_report.py EXPECTED OBSERVED")
    expected = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    observed = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
    compare(expected, observed)
    print("v0.2 reports match structurally and within the frozen floating-point allowance")


if __name__ == "__main__":
    main()
