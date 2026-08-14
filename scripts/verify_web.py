#!/usr/bin/env python3
"""Fail fast when the static laboratory loses required research or accessibility elements."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HTML = (ROOT / "web" / "index.html").read_text(encoding="utf-8")
JS = (ROOT / "web" / "js" / "app.js").read_text(encoding="utf-8")
CSS = (ROOT / "web" / "css" / "style.css").read_text(encoding="utf-8")

REQUIRED_HTML = (
    'href="#main"',
    'id="main"',
    'id="method"',
    'id="function"',
    'id="first"',
    'id="second"',
    'id="tolerance"',
    'id="run"',
    'id="trace-chart"',
    'aria-live="polite"',
    "EVIDENCE BOUNDARY",
    "PRIMARY SOURCES",
)
for needle in REQUIRED_HTML:
    assert needle in HTML, f"missing HTML contract: {needle}"

assert "solve_json" in JS
assert "requestAnimationFrame" not in JS, "motion must remain user-driven"
assert "@media (prefers-reduced-motion: reduce)" in CSS
assert "overflow-x: hidden" not in CSS, "do not conceal layout overflow"
print("web structure, research boundary, and accessibility contracts are present")
