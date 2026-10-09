# D52 — Portable Vision text-revision support value

## Objective

Add one portable, framework-owned value for the result of a Vision text-recognition revision
membership query. This does not claim portable image analysis or execution.

## Scope

- `crates/framework-vision/**`
- this plan

The orchestrator owns integration with shared capability status, global documentation, CI, and
support-count prose. Do not edit root manifests, CI, indexes, other matrix rows, platform backends,
Swift sources, or tests.

## Contract

- Expose `TextRecognitionRevisionSupport` with a system constructor, queried revision accessor, and
  `is_supported()` accessor.
- Keep the crate `#![no_std]`, unsafe-free, and free of Apple platform types.
- Interpret the boolean only as membership in `VNRecognizeTextRequest.supportedRevisions`.
- Do not model image recognition, supported languages, model readiness, accuracy, authorization, or
  execution success.

## Acceptance

- The portable contract builds with `--no-default-features` and has complete rustdoc.
- The portable guide states the exact partial scope and links to the iOS query guide.
- No tests, image processing, device execution, or performance claims are added.

## Status

Implemented. Local no-std, format, strict Clippy, and documentation evidence is recorded in
`PLAN_IOS_VISION.md`. No tests or Vision runtime query were run.
