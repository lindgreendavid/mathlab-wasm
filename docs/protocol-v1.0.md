# Frozen protocol — residual, forward error, and conditioning v1.0

**Frozen:** 2026-08-14, before implementing the v1.0 conditioning report and interactive
diagnostic.

## Research question

For prespecified scalar equations and candidate roots, when does a small absolute residual
`|f(x̂)|` track absolute forward error `|x̂ − r|`, and when can scaling or root multiplicity make
the residual misleading if it is interpreted alone?

## Scope and design

This is a deterministic educational verification of established numerical-analysis relationships,
not a sample of applied equations and not a claim of new mathematics. The unit of analysis is one
fixed equation–candidate pair evaluated with Rust `f64` arithmetic. Solver choice and timing are
outside this protocol: the candidate, reference root, function scale, and derivative are fixed
before the report is generated.

The unchanged v0.1 solver study and v0.2 safeguarded-method extension remain separately versioned.
Version 1.0 adds a conditioning diagnostic; it does not rewrite either earlier register.

## Quantities

For a candidate `x̂` and fixed reference root `r`:

- **absolute residual:** `|f(x̂)|`;
- **absolute forward error:** `|x̂ − r|`;
- **simple-root absolute condition number for additive output perturbations:** `1 / |f′(r)|`;
- **first-order error estimate:** `|f(x̂)| / |f′(r)|`, reported only when `f′(r) ≠ 0`.

The condition number is tied to the explicitly stated additive perturbation model. It is not a
scale-free property of a written equation: multiplying `f` by a nonzero constant changes the raw
residual and this absolute condition number while leaving the root and forward error unchanged.
For a multiple root with `f′(r) = 0`, the simple-root linearization is singular; the report returns
no finite simple-root condition number or first-order estimate.

## Prespecified cases

All linear cases use `r = 1` and `x̂ = 1.000001`.

| ID | Function | Candidate | Expected diagnostic |
| --- | --- | --- | --- |
| `flat-scaled-linear` | `10⁻⁸(x − 1)` | `1.000001` | tiny residual, same forward error as the other linear cases, condition `10⁸` |
| `unit-linear` | `x − 1` | `1.000001` | residual and forward error agree, condition `1` |
| `steep-scaled-linear` | `10⁸(x − 1)` | `1.000001` | large residual, same forward error as the other linear cases, condition `10⁻⁸` |
| `simple-cubic-local` | `x³ − x − 2` | `r + 10⁻⁸`, where `r = 1.5213797068045676` | first-order estimate closely tracks the fixed forward error |
| `repeated-root` | `(x − 1)²` | `1.00001` | residual is `≈10⁻¹⁰`, forward error is `≈10⁻⁵`, simple-root diagnostic unavailable |

## Endpoints and acceptance criteria

Primary endpoint: whether the diagnostic classification matches the prespecified expectation.
Secondary endpoints are residual, forward error, derivative magnitude at the reference root,
condition number, and first-order estimate.

The v1.0 report passes only if:

1. all values are finite except the deliberately unavailable simple-root quantities, which are
   serialized as `null`;
2. the three scaled linear cases have identical computed forward error;
3. their residuals satisfy `flat < unit < steep` and their condition numbers satisfy
   `flat > unit > steep`;
4. for each linear case, the first-order estimate equals the computed forward error within
   `32 × ε × (1 + forward error)`;
5. for the cubic case, the relative difference between the first-order estimate and forward error
   is at most `1e-7`;
6. the repeated-root case has derivative magnitude zero, `null` condition/estimate, residual below
   `1e-9`, and forward error above `1e-6`;
7. report regeneration is byte-for-byte deterministic;
8. the browser reads the committed report and does not recompute scientific values in JavaScript.

If a criterion fails, it is published as a deviation rather than silently weakening this protocol.

## Interpretation boundary

The five examples demonstrate why a stopping rule or reported result should not interpret raw
residual alone. They do not estimate how often ill-conditioning occurs, validate a posteriori error
bounds for arbitrary nonlinear problems, or replace interval methods, backward-error analysis, or
problem-specific perturbation models. The cubic estimate is a local first-order approximation, not
a rigorous enclosure.

## Sources fixed for interpretation

- NIST Digital Library of Mathematical Functions, §3.8 and §3.8(vi), “Nonlinear Equations” and
  “Conditioning of Zeros”: <https://dlmf.nist.gov/3.8>
- Higham, N. J. (2002), *Accuracy and Stability of Numerical Algorithms*, second edition,
  Society for Industrial and Applied Mathematics: <https://doi.org/10.1137/1.9780898718027>
- IEEE 754-2019, *IEEE Standard for Floating-Point Arithmetic*:
  <https://doi.org/10.1109/IEEESTD.2019.8766229>
