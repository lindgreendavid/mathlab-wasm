# Changelog

All notable changes follow Semantic Versioning.

## [0.2.0] - 2026-08-14

### Added

- Prespecified Brent–Dekker-style safeguarded root-finding protocol.
- Bracket-preserving hybrid with secant, inverse-quadratic, and bisection step labels.
- Five-case deterministic v0.2 report with bracket-invariant and step-mix checks.
- Interactive safeguarded-method traces and a deliberately skewed fallback demonstration.

### Changed

- Research report, method notes, website, CI, citation metadata, and release documentation now
  distinguish the unchanged v0.1 foundation from the v0.2 extension.

## [0.1.0] - 2026-08-14

### Added

- Frozen root-finding protocol and primary-source registry.
- Tested bisection, Newton, and secant implementations in Rust.
- Structured convergence traces and explicit failure statuses.
- Deterministic machine-readable benchmark report.
- Responsive, keyboard-accessible WebAssembly laboratory.
- CI, CodeQL, dependency review, and GitHub Pages workflows.
