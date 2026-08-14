# Mathlab WASM v1.0.0

Mathlab WASM is now a stable, reproducible research product. The new Residual Microscope makes a
foundational numerical-analysis distinction interactive: raw residual, forward error, and
conditioning are related, but they are not interchangeable.

## Included

- Five cases frozen before implementation in `docs/protocol-v1.0.md`.
- A Rust-generated, machine-readable conditioning report with explicit acceptance checks.
- Interactive comparison of residual, forward error, derivative, condition number, and local
  first-order estimate.
- Transparent handling of a repeated root where the simple-root diagnostic is unavailable.
- Preserved v0.1 solver and v0.2 safeguarded-method studies.

All prespecified v1.0 checks pass. The release remains an educational verification of established
mathematics, not novel theory, a universal stopping rule, or a production-solver certification.
