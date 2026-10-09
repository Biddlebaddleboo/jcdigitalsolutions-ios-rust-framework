# PLAN_IOS_NETWORK.md — Workstream B3: Foreground iOS HTTP Backend

## Status

B3 passes locked device/simulator checks and strict all-target Clippy. Its Release link/import gate
builds a probe for each target; `otool -L` reports exactly `Foundation`, `libSystem.B.dylib`, and
`libobjc.A.dylib`; the script also rejects Swift source and its selected Swift/Python/runtime and
unrelated capability symbol patterns. The probes are link-only and were not executed. The recorded
host unit test gate, `cargo test --locked -p ios-network`, passes 10 deterministic tests for
conversion and operation completion/drop races. No URLSession request, local-server differential,
or HTTP parity result is claimed; the recorded Xcode 26.6 host is below the plan's 27.x baseline.

## Next-scope gate

No additional B3 implementation is justified by the current plans. The adjacent E1 review in
[`PLAN_REPLACEMENTS_HTTP.md`](PLAN_REPLACEMENTS_HTTP.md) and
[`the HTTP construction decision`](docs/decisions/http-request-construction.md) traces portable
request validation and owned staging through the required Foundation `NSURLRequest` and URLSession
boundary; it identifies no bounded Apple operation that can be replaced without changing the
contract and no credible performance hypothesis. Reopen implementation only after naming a
replaceable operation and an apples-to-apples workload meeting the criteria in
[`PLAN_REPLACEMENTS_HTTP.md#reopening-and-acceptance-criteria`](PLAN_REPLACEMENTS_HTTP.md#reopening-and-acceptance-criteria).
The present baseline is [`OwnedRequest::from_request`](platform/ios/ios-network/src/conversion.rs),
[`IosSendFuture::start` / `make_request`](platform/ios/ios-network/src/platform.rs), and
[`response_from_callback`](platform/ios/ios-network/src/platform.rs); Foundation remains required
for URLSession request submission and callback values. B13 background downloads and B14 file
adoption remain separate completed scopes, not extensions of B3.

## Remaining network-family gate after 07cd525

The implemented slices are B3 foreground URLSession HTTP, B13/B14 background file download and
adoption, D15/B18 informational path status, and D19/B24 outbound TLS-over-TCP. They are separate
contracts: path status is not endpoint reachability, and byte streams do not add HTTP, listeners, or
UDP. B64 SafariServices HTTPS presentation is a host-UI slice.

No source or static-gate change is justified by the current contracts. B3's local-fixture differential
has no Apple app/runner and the recorded Xcode 26.6 host is below the 27.x baseline; E1 also found no
replacement candidate. D19/B24 leaves listeners and UDP unplanned without a bounded consumer need.
The nearest network-adjacent product candidate is PushKit row 033; D70 blocks source work until a
real VoIP product accepts APNs registration/token lifecycle, host launch/delegate ownership, and
CallKit reporting. WeatherKit REST also needs a trusted server-signed token source and explicit typed
response/attribution ownership.

Re-entry: open the PushKit slice only after the product owner accepts that VoIP/APNs/CallKit contract.
Reopen B3 parity only with a named behavioral gap and an Apple runner that compares URLSession and a
candidate against the same deterministic local fixture under the required toolchain; see
[`PLAN_REPLACEMENTS_HTTP.md#reopening-and-acceptance-criteria`](PLAN_REPLACEMENTS_HTTP.md#reopening-and-acceptance-criteria).

## Objective

Implement the iOS foreground HTTP backend for the stable `framework-network` contract using public Apple APIs and no Swift source or mandatory executor.

## Dependencies

- Foundation A, including `framework-async`, is integrated.
- B iOS runtime substrate is integrated.
- D1 `framework-network` request/response and `HttpBackend` contracts are integrated.

## Read first

- `PLAN.md`
- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES.md`
- `PLAN_CAPABILITIES_APP_DATA.md`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- `crates/framework-network/**`
- `platform/ios/ios-runtime/**`

## Write scope

- `platform/ios/ios-network/**`
- `docs/ios/network.md`
- capability-specific iOS integration tests owned by the crate

Do not edit portable network contracts, root workspace configuration, `Cargo.lock`, shared runtime APIs, capability manifest, C bindings, or other iOS backends. The orchestrator owns dependency centralization, lockfile reconciliation, and shared manifest updates.

## Backend requirements

- Implement `HttpBackend` with compile-time selection and public Foundation/URLSession APIs.
- Keep the backend iOS-only; it must not add Apple types to the portable API or link Network/UI frameworks without need.
- Start the native request when the returned future is first polled, matching `framework-network`.
- Remain executor-neutral; do not require Tokio, async-std, a global executor, runtime discovery, or a process-global session registry.
- Preserve request method, URL, duplicate headers, header order, and body bytes as far as `URLSession` permits. Document native normalization or unsupported fields based on observed API behavior rather than claiming exact parity by assumption.
- Return HTTP status codes, including non-success status codes, as ordinary responses; preserve response header duplicates/order where the public API permits observation.
- Map stable `NSError` categories explicitly and preserve its signed code only when it fits the current optional `PlatformErrorCode(i32)` contract. The current shared error type has no domain field, so this workstream does not retain the domain or extend the portable contract.
- Own request/response bytes safely across the native callback. Document every required copy and the OS/native callback thread.
- Define future drop, native task cancellation/detachment, completion exactly-once, callback reentrancy, and thread-safety behavior. No callback may panic across Objective-C/C.
- Reject or explicitly document any portable input that `URLSession` cannot represent without unsafe or undocumented behavior.
- Do not add cookies, retries, background transfer, or a custom connection pool unless the contract explicitly opts in; document native defaults that still apply.
- Keep the native escape handle out of the portable contract and avoid process-global state.

## Validation and evidence

- Add deterministic unit coverage for request/response/error conversion and future completion/drop races using fake or controlled operations where possible.
- Check the crate on `aarch64-apple-ios` and `aarch64-apple-ios-sim`; run Clippy with warnings denied.
- Link minimal consumers for both targets and inspect imports to confirm only required public frameworks/runtime symbols are present, with no Swift runtime/source.
- Run `sh platform/ios/ios-network/check-link-imports.sh` on macOS; it builds a link-only probe for both targets, checks exact direct imports `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` with `otool -L`, rejects the script's selected Swift/Python/runtime and unrelated capability symbol patterns with `nm -u`, and does not run either binary.
- Run applicable repository checks after dependency integration.
- Apple network differential tests must use deterministic local fixtures; do not use live external services or sleeps as correctness oracles. If the environment cannot run them, state the exact limitation and do not claim parity.
- No performance replacement is proposed; record measurement status as not measured.

## Documentation

Document URL and header semantics, HTTP status behavior, native defaults, permissions/privacy manifest and minimum OS facts only when verified, copies, callback thread, cancellation/drop semantics, errors/native codes, framework linkage, dependency substitution seam, examples, and tests not run. Do not claim networking parity or performance without real evidence.

## Non-goals

- No background transfers, low-level listener/socket API, reachability service, WebKit, Safari, or networking replacement candidate.
- No portable API redesign or executor integration.
- No Swift, Objective-C source, private API, or handwritten Apple ABI declarations.

## Handoff

Report changed files, commit SHA, exact checks and linkage imports, actual parity evidence, deviations, and unresolved assumptions. Link/import evidence is not URLSession runtime or parity evidence. The orchestrator integrates shared Cargo and capability-manifest changes.
