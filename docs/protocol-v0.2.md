# Frozen protocol — safeguarded root finding v0.2

**Frozen:** 2026-08-14, before implementation of the safeguarded solver and before executing the
v0.2 benchmark generator.

## Research question

Across a prespecified set of continuous, sign-changing scalar equations, can a Brent–Dekker-style
hybrid retain an enclosing bracket while accepting secant or inverse-quadratic interpolation when
the safeguard permits it, and report rejected inputs without overstating what a small benchmark
establishes?

## Scope and design

This is a deterministic verification of established numerical behavior, not novel mathematics and
not a representative solver benchmark. The unit of analysis is one method–scenario run under Rust
`f64` arithmetic. The implementation will be written after this protocol is committed.

The v0.1 bisection, Newton, and secant protocol and its machine-readable report remain unchanged.
Version 0.2 adds a separately versioned report for a bracket-preserving hybrid. Results from the new
method will not be used to rewrite the v0.1 hypotheses or outcomes.

## Safeguarded method

The new method will:

1. require finite, ordered endpoints whose function values have opposite signs, while accepting an
   exact zero at either endpoint;
2. maintain a sign-changing bracket for every nonterminal iteration;
3. propose a secant step when two distinct function values are available;
4. propose inverse-quadratic interpolation when three distinct function values are available;
5. accept interpolation only when the Brent-style size and progress safeguards hold;
6. otherwise take a bisection step;
7. report the accepted step kind, bracket endpoints, bracket width, iterate, residual, and function
   evaluation count in the trace;
8. stop on the frozen residual or bracket-half-width tolerance, a non-finite value, or the iteration
   budget.

This project uses an independently written pedagogical implementation. It does not claim bitwise
equivalence with Netlib, SciPy, or another production library.

## Common numerical policy

- arithmetic: Rust `f64` / IEEE-754 binary64 semantics;
- absolute `x` tolerance: `1e-10`;
- residual tolerance: `1e-10`;
- maximum iterations: `80`;
- interpolation denominators are guarded at `64 × ε × (1 + local scale)`;
- the active bracket is the certificate exposed by the method; a small residual alone is not
  described as a universal root-error bound;
- no timing endpoint is collected.

## Prespecified scenarios and expectations

| ID | Function | Interval | Expected qualitative result |
| --- | --- | --- | --- |
| `cubic-safeguarded` | `x³ − x − 2` | `[1, 2]` | converged within `1e-9` of the fixed reference root |
| `cosine-safeguarded` | `cos(x) − x` | `[0, 1]` | converged within `1e-9` of the fixed reference root |
| `skewed-safeguarded` | `x¹⁰ − 1` | `[0, 2]` | converged within `1e-9` of 1 with both interpolation and bisection visible in the trace |
| `endpoint-safeguarded` | `x³` | `[0, 1]` | exact endpoint root accepted without an iterative update |
| `repeated-safeguarded` | `(x − 1)²` | `[0, 2]` | invalid bracket despite the even-multiplicity root at 1 |

Fixed reference roots are `1.5213797068045676` for the cubic equation,
`0.7390851332151607` for the cosine equation, and `1.0` for the other three functions where a
reference is needed.

## Endpoints

Primary: categorical solver status. Secondary: absolute residual, iterations, function evaluations,
accepted step kind, bracket endpoints, and bracket width. Step counts are descriptive for these
fixed cases only; no aggregate “best method” score or production-performance claim will be made.

## Acceptance criteria

- every observed status matches the frozen expectation;
- each converged non-endpoint estimate differs from its fixed reference by less than `1e-9`;
- every recorded nonterminal bracket contains the fixed reference root and has non-increasing
  width, up to a `16 × ε × (1 + previous width)` comparison allowance;
- the skewed case contains at least one accepted interpolation step and at least one bisection
  fallback;
- the endpoint case returns the exact endpoint with two initial function evaluations and zero
  iterative updates;
- report regeneration is byte-for-byte deterministic;
- UI values and step labels come from the Rust/WASM result and are not recomputed in JavaScript.

If an expectation fails, the result will be reported as a deviation. The protocol will not be
silently changed to make a failed result pass.

## Evidence boundary

The five cases are intentionally selected demonstrations. They do not estimate failure prevalence,
prove superiority over bisection, reproduce a production library, or cover discontinuous,
multiple-root, complex-valued, or noisy functions beyond the explicitly rejected bracket example.

## Amendments

### 2026-08-14 — cross-platform transcendental serialization

**Timing:** added after implementation and after the macOS result was known, when the first Ubuntu
CI run compared the generated report with the committed file.

**Reason:** the Ubuntu and macOS system math libraries returned a last-bit difference for a cosine
residual. All statuses, reference-root decisions, bracket checks, step kinds, counts, and frozen
tolerance decisions agreed, but the complete JSON files differed at the exact decimal encoding of
that floating-point value. IEEE-754 binary64 does not require different elementary-function library
implementations to return bit-identical transcendental results.

**Change:** byte-for-byte regeneration remains required on the producing platform and remains the
gate for the unchanged v0.1 report. Cross-platform v0.2 CI now requires identical JSON structure,
keys, strings, booleans, integer counts, and list lengths; floating-point leaves must agree within
`16 × ε × (1 + max(|expected|, |observed|))`. The semantic acceptance checks are still recomputed
independently and must all pass.

**Effect on confirmatory status:** the five qualitative hypotheses and prespecified numerical
tolerances remain confirmatory. Cross-platform byte identity is withdrawn as an unsupported
technical claim and is not counted as a confirmed outcome.
