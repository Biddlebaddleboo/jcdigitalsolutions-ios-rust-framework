# PLAN_REPLACEMENTS_HTTP.md — Workstream E1: HTTP Construction Candidate Review

## Objective

Decide whether the existing Rust foreground-HTTP request construction has a bounded Apple behavior it can replace with a provable performance win.

## Dependencies

Requires `PLAN_FOUNDATION.md`, `PLAN_VALIDATION_HARNESS.md`, the D1 `framework-network` contract, and integrated B3 `ios-network` backend.

## Write scope

- `docs/decisions/http-request-construction.md`
- `benchmarks/replacements/http-construction/**` only if an apples-to-apples workload is identified
- candidate-specific parity fixtures only if a concrete candidate is justified

Do not change public Rust APIs, `ios-network`, default iOS behavior, or dependencies in this review.

## Required review

- Trace what `framework-network` validates and stores versus what B3 must construct through Foundation for `URLSession`.
- Identify the exact Apple behavior a Rust candidate would replace and the semantic subset it would claim.
- Do not treat borrowed Rust request values as equivalent to a Foundation `NSURLRequest` that can be submitted to `URLSession`.
- If the current design has no replaceable Apple work or no credible performance hypothesis, record a no-candidate/defer decision with code evidence and add no benchmark or production implementation.
- If a candidate is viable, add only a workload that compares the same inputs, output/error contract, and relevant setup cost. Use existing harnesses; host/simulator timing is advisory, not device evidence.
- Keep Apple behavior as default unless Apple differential parity passes and Release measurements on representative physical Apple hardware show a meaningful win without unacceptable regressions.

## Validation and handoff

- Run `cargo xtask docs-check` and `git diff --check`.
- If a harness workload is added, run its deterministic fixtures and report timing context and unmeasured dimensions.
- Report the Apple baseline, candidate scope or rejection, parity/performance evidence, default decision, changed files, commit SHA, and unresolved assumptions.
