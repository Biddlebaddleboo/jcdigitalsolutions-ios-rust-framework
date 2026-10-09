# PLAN_REPLACEMENTS_HTTP.md — Workstream E1: HTTP Construction Candidate Review

## Status

E1 is complete as a no-candidate/defer decision in
[`docs/decisions/http-request-construction.md`](docs/decisions/http-request-construction.md). The
review traces portable validation, owned request staging, Foundation request creation, and the
URLSession transport/response boundary. It finds no bounded Apple behavior that a Rust replacement
can remove while preserving the current contract and supporting a credible performance hypothesis.
Foundation and URLSession remain the default; no candidate benchmark, parity suite, API, or
dependency was added. Performance remains unmeasured. `cargo +1.94.1 xtask docs-check` and
`git diff --check` passed for this integrated checkout.

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

## Reopening and acceptance criteria

E1 has no candidate, so these gates are not authorization to implement a Rust HTTP stack or
optimize the current adapter. A later proposal must name the exact Foundation/URL Loading System
operation it removes and show why URLSession does not still require that operation. The current
baseline is the portable [`HttpRequest` and `HttpBackend`](crates/framework-network/src/lib.rs),
[`OwnedRequest::from_request`](platform/ios/ios-network/src/conversion.rs),
[`IosSendFuture::start` and `make_request`](platform/ios/ios-network/src/platform.rs), and
[`response_from_callback`](platform/ios/ios-network/src/platform.rs). The Apple boundary is
[`URLSession data-task request and callback`](https://developer.apple.com/documentation/foundation/urlsession/datatask%28with%3Acompletionhandler%3A%29-e6xv)
and [`NSURLRequest`](https://developer.apple.com/documentation/foundation/nsurlrequest?changes=_2);
the current behavior for repeated headers and observed response headers is bounded by
[`NSMutableURLRequest.addValue`](https://developer.apple.com/documentation/foundation/nsmutableurlrequest/addvalue%28_%3Aforhttpheaderfield%3A%29)
and [`NSHTTPURLResponse.allHeaderFields`](https://developer.apple.com/documentation/foundation/httpurlresponse/allheaderfields).

Before performance work, the proposal must define one deterministic local fixture and require
differential parity for the same request inputs, HTTP status, exposed response headers/body, error
categories/codes, cancellation/drop outcome, and session policies. The baseline and candidate must
use the same physical device model, iOS build, Rust target/toolchain, Xcode toolchain, Release
settings, fixture, and ATS/privacy state. The B3 baseline uses its documented default URLSession
configuration. A candidate that keeps URLSession must use that same configuration; a candidate that
changes transport must enumerate and match the relevant observable cache, cookie, credential,
redirect, proxy, trust, timeout, and cancellation behavior, or parity fails for that dimension.
The measurement plan must state whether setup is included, measure both the targeted operation and
end-to-end request cost, and predeclare the minimum improvement and acceptable regression bounds.
Interleave or randomize A/B runs and report sample count and uncertainty; host or Simulator timing
alone does not qualify.
Acceptance requires parity first, then a representative-device Release win beyond measurement
uncertainty and the predeclared threshold, with no exceeded regression bound. Only then may a
separate approval change the default. An adapter-only allocation optimization that removes no
Apple operation needs its own named scope and the same end-to-end evidence; it is not an E1
replacement candidate.

## Validation and handoff

- Run `cargo xtask docs-check` and `git diff --check`.
- If a harness workload is added, run its deterministic fixtures and report timing context and unmeasured dimensions.
- Report the Apple baseline, candidate scope or rejection, parity/performance evidence, default decision, changed files, commit SHA, and unresolved assumptions.
