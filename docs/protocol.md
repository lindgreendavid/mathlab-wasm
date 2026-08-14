# Frozen protocol — root-finding v0.1

**Frozen:** 2026-08-14, before executing the committed benchmark generator.

## Research question

Across a prespecified one-dimensional benchmark suite, which convergence guarantees and failure
modes of bisection, Newton, and the secant method are observable under the same IEEE-754 binary64
arithmetic, tolerances, and iteration budget?

## Scope and design

This is a deterministic verification of established numerical behavior. It is neither a random
sample of equations nor a claim of new mathematics. The unit of analysis is one method–scenario
run. All functions, parameters, success criteria, and expected qualitative outcomes are fixed below.

## Algorithms

1. **Bisection:** requires ordered endpoints with opposite signs. It preserves the sign-changing
   bracket and uses its midpoint.
2. **Newton:** uses the analytic derivative and the update `x[n+1] = x[n] − f(x[n])/f′(x[n])`.
3. **Secant:** replaces the derivative by the slope through the two latest iterates.

## Common numerical policy

- arithmetic: Rust `f64` / IEEE-754 binary64 semantics;
- `x` tolerance: `1e-10`;
- residual tolerance: `1e-10`;
- maximum iterations: `80`;
- Newton success requires residual and step tolerance, except the initial point where no step exists;
- secant success requires residual and step tolerance;
- bisection success requires residual tolerance or certified half-width tolerance;
- derivative and secant denominators below `64 × ε × (1 + local scale)` are not divided through;
- non-finite values, cycles, invalid brackets, and exhausted budgets are reported, never converted
  into successful results.

## Prespecified scenarios and expectations

| ID | Function | Method / inputs | Expected qualitative result |
| --- | --- | --- | --- |
| `cubic-bisection` | `x³ − x − 2` | bisection `[1, 2]` | converged within interval-halving bound |
| `cubic-newton` | `x³ − x − 2` | Newton `x₀=1.5` | converged in at most 5 iterations |
| `cosine-secant` | `cos(x) − x` | secant `0, 1` | converged without derivative |
| `repeated-bisection` | `(x−1)²` | bisection `[0, 2]` | invalid bracket despite root at 1 |
| `repeated-newton` | `(x−1)²` | Newton `x₀=2` | converged more slowly than simple-root Newton |
| `newton-cycle` | `x³−2x+2` | Newton `x₀=0` | detected `0 → 1 → 0` cycle |
| `secant-collapse` | `(x−1)²` | secant `0, 2` | equal function values collapse slope |

## Endpoints

Primary: categorical solver status. Secondary: absolute residual, iterations, function evaluations,
last step size, and final bracket width where defined. No aggregate “best solver” score will be
computed.

## Acceptance criteria

- every observed status matches the frozen expectation;
- converged simple-root estimates differ from their high-precision references by less than `1e-9`;
- bisection iterations do not exceed `ceil(log2((b−a)/x_tolerance))`;
- report regeneration is byte-for-byte deterministic;
- UI values are read from the Rust/WASM result, not recomputed independently in JavaScript.

## Amendments

None at freeze. Any later change must be appended with date, reason, and whether results were known.
