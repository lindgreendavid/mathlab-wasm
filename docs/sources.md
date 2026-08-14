# Source registry and claim map

Sources were checked on 2026-08-14. Primary and authoritative sources are preferred.

| Source | Identity | Used for | Boundary |
| --- | --- | --- | --- |
| NIST DLMF §3.8 | <https://dlmf.nist.gov/3.8> | Newton update, local quadratic convergence for simple roots, bisection sign-change premise and slow convergence | DLMF does not validate this implementation |
| Brent (1971) | DOI <https://doi.org/10.1093/comjnl/14.4.422> | Context for safeguarded combinations of interpolation and bisection | Brent's method is cited but not implemented in v0.1 |
| IEEE 754-2019 | DOI <https://doi.org/10.1109/IEEESTD.2019.8766229> | Floating-point formats, operations, and exception context | Access to the standard may require subscription; no conformance certification is claimed |
| Rust `f64` documentation | <https://doc.rust-lang.org/std/primitive.f64.html> | Binary64 constants and finite-value behavior used by safeguards | Language documentation is not a numerical-method proof |
| `wasm-bindgen` documentation | <https://docs.rs/wasm-bindgen/latest/wasm_bindgen/> | Typed Rust-to-WebAssembly browser interface | Tooling source only |

## Claim discipline

- “Guaranteed” is used only for bisection under an ordered, continuous, sign-changing bracket and
  refers to interval containment/halving, not arbitrary inputs.
- “Quadratic” is described as local behavior near a simple root, not an unconditional Newton rate.
- No benchmark scenario is treated as a representative sample of scientific computing workloads.
- Failure examples demonstrate possibility, not prevalence.
