# PLAN_IOS_PRESENTATION.md — Workstream B9: UIKit Acknowledgement Alert

## Objective

Add one bounded, system-owned UIKit presentation path: a one-action acknowledgement alert from a caller-owned `UIViewController`. Keep it separate from `ios-sharing` and make no claim of a general navigation, modal, action-sheet, or presentation framework.

## Dependencies

- Foundation A and the `ios-runtime` main-thread proof are integrated
- Public `objc2-ui-kit` bindings expose `UIAlertController`, `UIAlertAction`, and `UIViewController` presentation APIs

## Read first

- `PLAN_IOS_NATIVE.md`
- `PLAN_IOS_SHARE.md`
- `PLAN_IOS_ACCESSIBILITY.md`
- `docs/ios/runtime.md`
- `docs/ios/sharing.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/APP_STORE_COMPLIANCE.md`
- `docs/VALIDATION.md`

## Write scope

- `platform/ios/ios-presentation/**`
- `docs/ios/presentation.md`

Do not edit `ios-sharing`, portable contracts, root workspace/dependency declarations, `Cargo.lock`, the capability manifest, CI, or other iOS backends. The orchestrator owns Cargo.lock and capability/validation documentation integration.

## Required surface

- Add an `ios-presentation` crate with a synchronous, main-thread-bound API to create and present a `UIAlertController` with alert style and exactly one default acknowledgement action.
- Accept a caller-owned borrowed `UIViewController` presenter and borrowed UTF-8 title, message, and action text. Do not create or retain an application, window, presenter, or view tree.
- Do not expose a callback, result future, native controller handle, arbitrary action list, action sheet, popover, custom controller, SwiftUI, or generic navigation API.
- Use only public UIKit declarations through generated `objc2` bindings. The action handler may be absent; UIKit owns the alert after presentation and the default action only dismisses it.
- Require the `ios-runtime::main_thread::MainThread` proof and keep UIKit state `!Send`/`!Sync`.
- Preflight only conditions that can be checked synchronously without loading a view, such as a missing loaded view/window or an active presentation/dismissal. Return bounded errors for known-invalid context.
- Document that `presentViewController` returns `void` and has no error callback; returning from the call only means a request was issued, not that the alert appeared or completed. Make no recovery claim for unforeseen UIKit refusal or a race after preflight.
- Record the public API floor from SDK declarations and distinguish it from the installed SDK deployment minimum. Do not set a crate deployment target unless required by the code.
- State that the crate is a partial system-UI backend and that the caller owns presentation context and lifecycle.

## Validation and handoff

- Add focused host tests only for pure conversion/error behavior that exists in the crate; do not simulate UIKit.
- Run device and simulator `cargo check` and strict all-target Clippy, formatting, docs-check, zero-Swift-source, and diff checks.
- If a temporary external consumer link/import check is used, report exact deployment targets and imports; no live alert UI, dismissal, device, or VoiceOver behavior is claimed.
- Report changed files, commit SHA, exact checks/results, API floor evidence, deviations, and unresolved assumptions. Do not push.
