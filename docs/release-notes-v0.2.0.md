# Mathlab WASM v0.2.0

This release adds a prespecified, bracket-preserving Brent–Dekker-style root finder to the existing
Rust/WebAssembly laboratory.

## Included

- Secant and inverse-quadratic proposals with explicit bisection fallback.
- Bracket endpoints, width, and accepted step kind in every safeguarded trace.
- Five frozen scenarios, including a skewed `x¹⁰−1` case that exercises all three step kinds.
- Machine-readable acceptance checks tied to the pre-implementation protocol commit.
- Updated interactive laboratory, research report, source registry, accessibility checks, and CI.

## Frozen result

All five v0.2 expectations pass. The result is a bounded deterministic verification of established
behavior, not a representative performance benchmark or a claim of equivalence with Netlib, SciPy,
or another production solver.
