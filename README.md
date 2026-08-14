# Mathlab WASM

**[Open the interactive laboratory →](https://lindgreendavid.github.io/mathlab-wasm/)**

[Read the v0.2 protocol](docs/protocol-v0.2.md) · [Inspect the v0.2 results](reports/v0.2-safeguarded-root-finding.json) · [View releases](https://github.com/lindgreendavid/mathlab-wasm/releases)

An inspectable Rust/WebAssembly laboratory for understanding when one-dimensional root-finding
methods converge, what their stopping rules certify, and how they fail. The laboratory compares
bisection, Newton, secant, and a bracket-preserving Brent–Dekker-style hybrid across separately
frozen versioned suites.

## Status

**Research product v0.2.0 — prespecified safeguarded-method extension.** The unchanged v0.1
foundation remains reproducible. This is an educational verification of established
numerical-analysis behavior, not novel mathematics or a general solver ranking.

## Fixed question

> Across a prespecified one-dimensional benchmark suite, which convergence guarantees and failure
> modes of bisection, Newton, and the secant method are observable under the same binary64
> arithmetic, tolerances, and iteration budget?

The primary endpoint is the solver status. Secondary endpoints are absolute residual, iteration
count, function-evaluation count, step size, and—where defined—bracket width.

## What v0.1 demonstrates

- Bisection converges for every continuous, sign-changing bracket in the frozen suite within its
  theoretical interval-halving bound.
- Newton converges rapidly near the selected simple roots but enters the prespecified `0 → 1 → 0`
  cycle for `f(x) = x³ − 2x + 2` from `x₀ = 0`.
- Bisection rejects the repeated root of `(x − 1)²` when the supplied endpoints do not change sign;
  that is a limitation of the bracket test, not evidence that no root exists.
- The secant method reaches the selected simple root without a derivative, but a zero difference in
  function values is detected rather than divided through.

These are bounded demonstrations. They do not establish performance on arbitrary functions,
finite-precision platforms, or production solver libraries.

## What v0.2 adds

- A sign-changing bracket is retained while secant and inverse-quadratic steps are proposed.
- Unsafe or insufficiently progressive interpolation falls back visibly to bisection.
- The skewed `x¹⁰−1` case exercises secant, inverse-quadratic, and bisection steps in one trace.
- Every recorded v0.2 bracket contains its fixed reference root and has non-increasing width within
  the frozen floating-point comparison allowance.
- The exact endpoint and invalid-bracket cases keep successful termination distinct from rejected
  assumptions.

The v0.2 protocol was published at commit `e4c6f222c22f163b909503d05ead800394757f26`
before the new implementation and result run. Its dated amendment transparently records why
cross-platform transcendental results are compared within a `16 × ε`-scaled allowance rather than
claimed to be byte-identical.

## Reproduce

```bash
cargo test --locked
cargo run --example generate_report -- reports/v0.1-root-finding.json
cargo run --example generate_v0_2_report -- reports/v0.2-safeguarded-root-finding.json
wasm-pack build --target web --out-dir web/pkg
python3 scripts/verify_report.py
python3 scripts/verify_web.py
```

Serve `web/` over HTTP after the WebAssembly build:

```bash
python3 -m http.server 8080 --directory web
```

Then open `http://127.0.0.1:8080/`.

## Repository map

| Path | Purpose |
| --- | --- |
| `src/lib.rs` | Rust solver implementations, trace schema, and WASM export |
| `tests/solver_tests.rs` | Convergence, bound, and failure-mode regression tests |
| `docs/protocol.md` | Frozen question, hypotheses, endpoints, tolerances, and exclusions |
| `docs/protocol-v0.2.md` | Pre-implementation safeguarded-method protocol |
| `docs/methods.md` | Algorithms, stopping rules, numerical safeguards, and limitations |
| `docs/sources.md` | Primary-source registry and claim-to-source map |
| `docs/v0.1-release-audit.md` | Release gate, completed checks, and remaining limits |
| `reports/v0.1-root-finding.json` | Machine-readable benchmark result |
| `reports/v0.2-safeguarded-root-finding.json` | Machine-readable v0.2 result and acceptance checks |
| `web/` | Accessible interactive laboratory |

## Evidence boundaries

- The examples were chosen to expose known behavior; this is not a blinded or representative sample
  of nonlinear equations.
- `f64` results are platform-relevant demonstrations, not proofs over real arithmetic.
- A small residual alone need not imply a small root error for an ill-conditioned zero.
- Iteration and evaluation counts depend on the exact stopping rules and safeguards documented here.
- The methods are pedagogical implementations and do not replace mature numerical libraries.
- “Brent–Dekker-style” identifies an algorithm family and does not claim bitwise equivalence with
  Netlib, SciPy, or another production implementation.

## Primary sources

- NIST Digital Library of Mathematical Functions, §3.8, “Nonlinear Equations”: <https://dlmf.nist.gov/3.8>
- Brent, R. P. (1971), “An Algorithm with Guaranteed Convergence for Finding a Zero of a Function”: <https://doi.org/10.1093/comjnl/14.4.422>
- IEEE 754-2019, *IEEE Standard for Floating-Point Arithmetic*: <https://doi.org/10.1109/IEEESTD.2019.8766229>
- `wasm-bindgen` guide and API documentation: <https://docs.rs/wasm-bindgen/latest/wasm_bindgen/>

## License

Code and original prose are MIT-licensed. Citations and linked works retain their original rights.
