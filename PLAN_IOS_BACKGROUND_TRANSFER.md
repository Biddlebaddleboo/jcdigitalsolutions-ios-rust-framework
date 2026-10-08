# PLAN_IOS_BACKGROUND_TRANSFER.md — Workstream B13: Background HTTP File Downloads

## Status

Proposal only. Do not implement B13 until D10 approves the portable durable-transfer contract below. B13 does not extend or alter B3's foreground `HttpBackend`.

## Objective

Add one iOS backend for OS-managed background HTTP(S) file downloads using Foundation `URLSessionConfiguration.background(withIdentifier:)`. Keep the response body file-backed, expose job state through the portable `framework-transfer` contract, and reattach to the same session after an OS relaunch.

Apple documents background URL sessions as transfers performed by a separate process. The app can recreate a session with the same identifier to reattach, then process delegate events before it invokes UIKit's background-session completion handler. A user force-quit cancels background transfers and prevents automatic relaunch. See [URLSessionConfiguration.background(withIdentifier:)](https://developer.apple.com/documentation/foundation/urlsessionconfiguration/background%28withidentifier%3A%29) and [UIApplicationDelegate.application(_:handleEventsForBackgroundURLSession:completionHandler:)](https://developer.apple.com/documentation/uikit/uiapplicationdelegate/application%28_%3Ahandleeventsforbackgroundurlsession%3Acompletionhandler%3A%29).

## Dependencies and gates

- A and D1 are integrated: `framework-core`, `framework-files::AppPath`, and `framework-network::{HttpUrl, Header, ResponseHeader, StatusCode}` exist.
- D10 is approved and implemented as a new `framework-transfer` portable contract. D10 must define stable job identity, duplicate-ID behavior, durable status query after process relaunch, terminal-state retention/removal, cancellation-request semantics, HTTP response metadata, and atomic destination completion. A Rust future alone is insufficient because its state cannot survive process death.
- B14 is separately defined and completed by the orchestrator before B13 starts. B14 owns the secure URLSession-temp-file adoption primitive within `ios-files`; B1 owns all of `platform/ios/ios-files/**`, so B13 may not edit that crate.
- B3 remains a separate foreground HTTP implementation. Its `HttpBackend` returns an in-memory `HttpResponse` and does not define task IDs, durable job state, destination files, or relaunch handling.
- The existing `ios-files` path rules remain the only destination-path boundary. B13 consumes B14's operation to adopt a URLSession temporary download file at an `AppPath` without reading the body into a `Vec`.

## Dependency boundary

- New `ios-transfer` path dependencies: `framework-transfer` (D10), `framework-network` (request/response values), `framework-files` (`AppPath`), and `ios-files` (B14's secure file-adoption operation).
- iOS-target dependencies: existing workspace-pinned `objc2` and `objc2-foundation`; enable only the generated Foundation features needed for `NSURLSession`, its configuration/delegate/task/response types, URL/file URL conversion, strings, and response-header inspection. Use the generated delegate protocol surface, not URLSession completion-block task APIs.
- The app-delegate forwarding example may use the existing workspace-pinned `objc2-ui-kit` and `block2` to receive and retain UIKit's completion block. Keep UIKit and `block2` out of the portable D10 contract and out of `ios-transfer` unless the final forwarding API requires a crate-owned block adapter.
- No new third-party crate is proposed. If the installed generated binding lacks any required public declaration, stop and request a central dependency/binding review; do not add handwritten Apple ABI declarations or Swift source.

## D10 contract proposal for review

Create `crates/framework-transfer` as `#![no_std]` plus `alloc`, with static backend selection and no executor, global registry, callback requirement, or platform type in its public API. Proposed semantic surface:

- `TransferId(u128)`: caller-assigned, nonzero, stable across app launches.
- `DownloadRequest<'a>`: `TransferId`, an HTTP(S) `HttpUrl`, borrowed request headers, and an `AppPath<'a>` destination. First slice is GET only, with no request body, upload, progress stream, or redirect policy override.
- `TransferBackend`: synchronous `start_download`, `status`, `cancel`, and terminal-job `forget` operations. `start_download` requests OS work; it does not promise immediate network activity. Duplicate IDs with existing or retained state return a conflict and never silently replace another task.
- `TransferStatus`: queued, active, succeeded with HTTP status and observed response headers, failed with portable error detail, or cancelled. A succeeded status is visible only after the full file is atomically installed at the requested destination. HTTP non-success statuses remain ordinary completed HTTP results; transport and file errors are failures.
- Job state and terminal results remain queryable after process relaunch until `forget`. The contract must state whether `cancel` is a request or a guarantee; proposed semantics are request-only once the native task has started.
- A backend that cannot meet the durable job contract reports the capability unavailable rather than returning a weaker success.

These names and rules are a review sketch, not an approved D10 API. D10 must settle metadata persistence and its crash/relaunch boundary before B13 implementation. `framework-files::FileBackend` currently writes byte slices and has no file-adoption operation; B13 must not copy a large download through its in-memory `write` API.

## Write ownership after D10 approval

- `crates/framework-transfer/**`: portable transfer values and static contract; owned by D10, not B13.
- `platform/ios/ios-transfer/**`: new URLSession background-download backend, delegate state, stable session identifier, task-to-`TransferId` mapping, and reattachment. Owned by B13.
- `docs/ios/transfer.md`: API floor, background lifecycle/force-quit behavior, status retention, file-copy cost, native defaults, cancellation, callback queue, and limitations.
- `examples/ios-minimal/**`: optional Rust app-delegate forwarding example only; the consuming application owns its `UIApplicationDelegate` and remains responsible for forwarding UIKit's background URLSession event.

No `platform/ios/ios-files/**`, root Cargo manifest, lockfile, capability manifest, B3 source, notification source, Swift source, or other backend is owned by B13. B14 must be named and delegated by the orchestrator before implementation. The orchestrator owns shared dependency and manifest edits.

## iOS backend design

- Use only public Foundation `NSURLSessionConfiguration`, `NSURLSession`, `NSURLSessionDownloadTask`, and generated `NSURLSessionDownloadDelegate`/task delegate bindings. Keep the backend in `ios-transfer`; do not add Network.framework, UIKit, WebKit, SafariServices, or a Swift bridge to the transfer crate.
- Use a stable, app-unique background session identifier supplied at construction. The caller owns `IosTransferBackend` for the app lifetime and recreates it with the same identifier during launch. Do not add process-global session lookup or task registries.
- Use the Foundation delegate API with a nil delegate queue, which gives URLSession its serial delegate queue. Correlate tasks through `TransferId` metadata plus D10's durable record; never rely on process-local `taskIdentifier` alone.
- On `URLSession:downloadTask:didFinishDownloadingToURL:`, `ios-transfer` synchronously hands the temporary URL and destination to B14's `ios-files` operation before the delegate returns, as URLSession only guarantees that temporary location for the callback. B14 must reject unsafe destination paths, avoid a full-body memory copy, stage within the destination directory, and expose the final file only at an atomic commit point. A failed download or install must not report success or expose partial bytes.
- The Rust app delegate receives UIKit's `application(_:handleEventsForBackgroundURLSession:completionHandler:)`, matches the supplied identifier to its owned backend, and forwards the completion handler. The backend retains that handler until URLSession reports that all background events for the session are delivered, then invokes it exactly once. B13 does not create a replacement app delegate or depend on `UIApplication.sharedApplication`.
- The URLSession delegate callback queue, event-handler completion thread, reentrancy, exactly-once behavior, and cancellation races must be documented. Objective-C delegate boundaries must not unwind; callback state and native ownership must follow repository `objc2` rules.
- `NSURLResponse.allHeaderFields` cannot guarantee duplicate-header or source-order fidelity. Normalize the observed Foundation dictionary into D10's header values and document that backend difference. Do not claim parity with B3's request-header representation.
- Do not set a crate deployment target. Record the generated declaration API floor from the installed SDK during implementation; Apple currently documents background URL sessions as available from iOS 7.0. Verify the selected SDK before recording that floor as implementation evidence.

## Exclusions

- Uploads, including background uploads; request bodies; progress callbacks; custom retry or redirect policy; a generic connection pool.
- Foreground HTTP, low-level Network.framework connections/listeners, reachability, WebKit, in-app Safari, and external URL opening.
- B12 notification-response delivery or any notification work.
- Durable execution after a user force-quit. The OS may cancel transfers and suppress relaunch in that case; surface the resulting state without claiming continuation.
- A process-global event router, mandatory executor, Rust-managed `UIApplicationDelegate`, or persisted state that is not defined by D10.

## Validation after implementation approval

- Add focused portable contract tests in `framework-transfer` and crate-owned deterministic state/conversion tests; do not use live external endpoints or timing sleeps as correctness oracles.
- Check iOS device and simulator targets, strict Clippy, formatting, docs, and minimal-link imports. Inspect that B13 adds Foundation only and no Swift runtime or unrelated Apple framework.
- Exercise a controlled local download and relaunch/session-reattach behavior only where an available Apple runner can provide deterministic evidence. State when device lifecycle tests cannot run; do not claim OS lifecycle parity from compile checks.
- Report file adoption atomicity, memory-copy behavior, response header limitations, task cancel/drop behavior, session ownership, API floor evidence, exact checks, imports, parity evidence, deviations, and unresolved assumptions.
