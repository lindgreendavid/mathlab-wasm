# Methods and numerical safeguards

## What each method certifies

Bisection combines continuity with an endpoint sign change. Under those assumptions, each step
retains at least one root in the bracket, and the interval width halves. The implementation reports
that width directly. A same-sign interval is rejected; it is not classified as root-free because an
even-multiplicity root may touch zero without changing sign.

Newton's familiar quadratic convergence is local and applies to a simple root under regularity and
sufficiently favorable initialization. The implementation therefore labels a detected cycle, small
derivative, non-finite iterate, or exhausted iteration budget explicitly. It does not add a hidden
line search or bracket, because doing so would change the method being taught.

The secant method avoids analytic derivatives by using two function values. It can converge faster
than bisection near a regular root, but the denominator can collapse. That failure is an observed
status rather than an unchecked division.

The safeguarded method keeps a sign-changing bracket while proposing secant or inverse-quadratic
interpolation. A proposal is accepted only when it lies inside the protected portion of the bracket
and makes sufficient progress relative to recent steps; otherwise the method bisects. This is an
independently written Brent–Dekker-style teaching implementation, not a claim of bitwise equivalence
with Netlib or SciPy.

## Stopping rules

A solver can satisfy a small residual while the approximation remains far from an ill-conditioned
root. Newton and secant therefore require both residual and step tolerances. Bisection may instead
stop on its certified half-width. The lab presents both residual and geometric progress so users can
see that they answer different questions.

## Residual and conditioning diagnostic

For a simple reference root `r`, a first-order expansion gives
`f(x̂) ≈ f′(r)(x̂-r)`. Under the frozen additive function-value perturbation model, the local
absolute root condition number is `1/|f′(r)|`, and the associated first-order error estimate is
`|f(x̂)|/|f′(r)|`. This is a local approximation, not a rigorous interval bound.

Multiplying an equation by a nonzero constant leaves its roots and a fixed candidate's forward
error unchanged, but it rescales the raw residual. The condition number above changes inversely,
so the product of residual and condition remains aligned for the frozen linear cases. At a multiple
root, `f′(r)=0`; the simple-root linearization is singular and the diagnostic is reported as
unavailable. These definitions and their perturbation model follow the interpretation fixed in the
v1.0 protocol and the cited numerical-analysis sources.

## Floating-point boundary

All calculations use Rust `f64`. IEEE 754 standardizes binary floating-point formats and operations,
but finite precision still introduces rounding, overflow, underflow, and representation error. The
results are deterministic for the committed implementation and inputs; they are not symbolic proofs.
