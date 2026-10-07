# PLAN_IOS_NETWORK.md — Workstream B3: Foreground iOS HTTP Backend

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
- Preserve native `NSError` domain/code in the framework's native error detail supported by current shared contracts; map stable categories explicitly.
- Own request/response bytes safely across the native callback. Document every required copy and the OS/native callback thread.
- Define future drop, native task cancellation/detachment, completion exactly-once, callback reentrancy, and thread-safety behavior. No callback may panic across Objective-C/C.
- Reject or explicitly document any portable input that `URLSession` cannot represent without unsafe or undocumented behavior.
- Do not add cookies, retries, background transfer, or a custom connection pool unless the contract explicitly opts in; document native defaults that still apply.
- Keep the native escape handle out of the portable contract and avoid process-global state.

## Validation and evidence

- Add deterministic unit coverage for request/response/error conversion and future completion/drop races using fake or controlled operations where possible.
- Check the crate on `aarch64-apple-ios` and `aarch64-apple-ios-sim`; run Clippy with warnings denied.
- Link minimal consumers for both targets and inspect imports to confirm only required public frameworks/runtime symbols are present, with no Swift runtime/source.
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

Report changed files, commit SHA, exact checks and linkage imports, actual parity evidence, deviations, and unresolved assumptions. The orchestrator integrates shared Cargo and capability-manifest changes.
