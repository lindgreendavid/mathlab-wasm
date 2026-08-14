# Mathlab WASM

**[Open the interactive laboratory →](https://lindgreendavid.github.io/mathlab-wasm/)**

[Read the frozen protocol](docs/protocol.md) · [Inspect the v0.1 results](reports/v0.1-root-finding.json) · [View releases](https://github.com/lindgreendavid/mathlab-wasm/releases)

An inspectable Rust/WebAssembly laboratory for understanding when one-dimensional root-finding
methods converge, what their stopping rules certify, and how they fail. The first release compares
bisection, Newton, and secant iterations on a frozen suite of simple roots, repeated roots, flat
derivatives, and an explicit Newton two-cycle.

## Status

**Research product v0.1.0 — frozen methods and deterministic benchmark suite.** This is an
educational reproduction of established numerical-analysis results, not a claim of novel
mathematics and not a general ranking of solvers.

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

## Reproduce

```bash
cargo test --locked
cargo run --example generate_report -- reports/v0.1-root-finding.json
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
| `docs/methods.md` | Algorithms, stopping rules, numerical safeguards, and limitations |
| `docs/sources.md` | Primary-source registry and claim-to-source map |
| `docs/v0.1-release-audit.md` | Release gate, completed checks, and remaining limits |
| `reports/v0.1-root-finding.json` | Machine-readable benchmark result |
| `web/` | Accessible interactive laboratory |

## Evidence boundaries

- The examples were chosen to expose known behavior; this is not a blinded or representative sample
  of nonlinear equations.
- `f64` results are platform-relevant demonstrations, not proofs over real arithmetic.
- A small residual alone need not imply a small root error for an ill-conditioned zero.
- Iteration and evaluation counts depend on the exact stopping rules and safeguards documented here.
- The methods are pedagogical implementations and do not replace mature numerical libraries.

## Primary sources

- NIST Digital Library of Mathematical Functions, §3.8, “Nonlinear Equations”: <https://dlmf.nist.gov/3.8>
- Brent, R. P. (1971), “An Algorithm with Guaranteed Convergence for Finding a Zero of a Function”: <https://doi.org/10.1093/comjnl/14.4.422>
- IEEE 754-2019, *IEEE Standard for Floating-Point Arithmetic*: <https://doi.org/10.1109/IEEESTD.2019.8766229>
- `wasm-bindgen` guide and API documentation: <https://docs.rs/wasm-bindgen/latest/wasm_bindgen/>

## License

Code and original prose are MIT-licensed. Citations and linked works retain their original rights.
