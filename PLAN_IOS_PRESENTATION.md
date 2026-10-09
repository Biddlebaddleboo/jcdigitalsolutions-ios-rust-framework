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
- Use only public UIKit declarations through generated `objc2` bindings. Supply no action handler and no presentation-completion block; UIKit manages the alert only if it accepts and presents the request, and the default action only dismisses it.
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

## Status and evidence

Status: complete. The runtime implementation meets the bounded B9 surface; no runtime behavior correction was needed. This audit corrects source rustdoc to describe UIKit management only when it accepts and presents the request, and adds the temporary consumer's link/import evidence to the guide. `present_acknowledgement` requires `ios_runtime::main_thread::MainThread`, borrows the caller-owned `UIViewController` and UTF-8 strings, creates an alert-style `UIAlertController` with exactly one default `UIAlertAction` and no handler, and issues the UIKit presentation request. The crate creates no application, window, presenter, or view tree and exposes no callback, future, alert handle, or general presentation API. Its preflight checks presentation/dismissal transition state, an already-presented controller, `viewIfLoaded` without loading the view, and the loaded view's window; known-invalid states map to the four documented `PresentationError` variants. UIKit manages the presented alert only if it accepts and presents the request. The guide accurately states that `presentViewController:animated:completion:` returns `void` without an error callback, so success proves only that a request was issued; it claims no recovery from UIKit refusal or a hierarchy race. UIKit does provide an optional presentation-completion block called after the presented controller's `viewDidAppear:`; B9 passes no block, and the action handler is also absent, so B9 exposes no presentation-transition, selection, or dismissal signal.

On Xcode 26.6 (build 17F113), both iPhoneOS and iPhoneSimulator SDKs are 26.5. The installed iPhoneOS 26.5 public headers declare `UIAlertController` and `UIAlertAction` from iOS 8.0, `presentViewController:animated:completion:`, `presentedViewController`, `isBeingPresented`, and `isBeingDismissed` from iOS 5.0, and `UIViewController.viewIfLoaded` from iOS 9.0; therefore the effective public API floor is iOS 9.0. `SDKSettings.plist` separately reports iOS 12.0 as its minimum deployment target and 26.5 as its default. `platform/ios/ios-presentation/Cargo.toml` sets no deployment target. The repository's Xcode 27.x baseline is not met by this host. A temporary external Rust `cdylib` consumer invoked the B9 API and linked for device at deployment target iOS 12.0 and Simulator at iOS 14.0 against SDK 26.5; it used its own offline lockfile and target directory outside the checkout. `vtool -show-build` confirmed device minos 12.0 / SDK 26.5 and Simulator minos 14.0 / SDK 26.5. `otool -L` reports direct imports UIKit, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib` for both targets; `nm -u` showed `objc_msgSend` and `objc_retainAutoreleasedReturnValue`, with no Swift or Python runtime symbols. Neither linked consumer was executed, so this is link/import evidence only and makes no UIKit runtime claim.

The required checks passed in the worktree based on repository HEAD `7b5513fa2a4864d21a594cbf1fbd43951427155`. Other workstreams had in-progress shared-file changes in the worktree. This audit changed the B9 guide and plan evidence and corrected source rustdoc; runtime implementation behavior and the `ios-presentation` lockfile entry were unchanged:

- `cargo +1.94.1 check --locked -p ios-presentation --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-presentation --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 fmt --all -- --check`
- `cargo +1.94.1 xtask docs-check`
- `cargo +1.94.1 xtask zero-swift-source`
- `git diff --check`

The external consumer link/import probe built in Release mode for `aarch64-apple-ios` with
`IPHONEOS_DEPLOYMENT_TARGET=12.0` and `aarch64-apple-ios-sim` with
`IPHONEOS_DEPLOYMENT_TARGET=14.0`; `otool -L`, `nm -u`, and `vtool -show-build` inspection passed
for each unexecuted artifact.

Post-integration recheck: both B9 device/simulator checks, strict Clippy commands, crate-scoped
formatting, docs-check, zero-Swift-source, and diff-check passed. At that recheck, workspace-wide
`cargo +1.94.1 fmt --all -- --check` stopped on D19-owned in-progress files: rustfmt reported
changes in `crates/framework-connection/src/lib.rs`, then printed the exact error
``Error writing files: failed to resolve mod `platform`: /Users/john/Projects/jcdigitalsolutions-ios-rust-framework/platform/ios/ios-connection/src/platform.rs does not exist``.
That D19 file now exists, but this audit did not rerun workspace-wide formatting, so its current
status was unverified at that point. This audit reran
`cargo +1.94.1 fmt --all -- --check`; it passed. B9 did not edit either D19 path.

No host test was added or run because the crate has no pure conversion or error-mapping helper to test; UIKit preflight was not simulated. These checks establish compile/lint and static SDK evidence only. No app launch, live alert presentation or dismissal, user interaction, device execution, simulator runtime, or VoiceOver behavior was tested. No commit was created.
