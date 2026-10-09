# PLAN_IOS_SHARE.md — Workstream B7: iOS Outgoing Share UI Backend

## Status

B7's `IosShareBackend` and retained callback-based `IosShareSession` are present in the local
tree. The callback session is the B7 seam required by F7. Its completion path treats UIKit's
callback queue as unspecified, copies only scalar result data off-main, and dispatches callback
state access and UIKit cleanup asynchronously to main so notification follows `start` return.
Synchronous `IosShareSession::start` rejection returns the unconsumed callback in
`ShareStartError<F>`; request conversion and presentation preflight finish before callback
ownership transfers to the session. The host test
`synchronous_preflight_rejection_returns_the_unconsumed_callback` exercises the UIKit-free
preflight/callback handoff used by `start`, verifies rejection does not invoke the callback, then
invokes the returned callback. `callback_completion_detaches_before_notifying_exactly_once` and
`detached_callback_completion_suppresses_late_result` exercise the production callback completion
state used by session completion, cancel, and drop. These host tests do not instantiate the
target-gated UIKit session. No live share UI behavior is claimed.

Host tests passed: `cargo test --locked --offline -p ios-sharing` (14 tests total) and
`cargo test --locked --offline -p ios-sharing --no-default-features --features share` (10 tests).
Device and simulator checks passed:
`cargo check --locked --offline -p ios-sharing --no-default-features --features share --target aarch64-apple-ios`
and the same command with `aarch64-apple-ios-sim`. Strict Clippy passed for both targets with
`cargo clippy --locked --offline -p ios-sharing --no-default-features --features share --target aarch64-apple-ios -- -D warnings`
and the same command with `aarch64-apple-ios-sim`. Also passed:
`cargo fmt --all -- --check`,
`cargo doc --locked --offline -p ios-sharing --no-deps --features share --target aarch64-apple-ios`,
`cargo run --locked --offline -p xtask -- docs-check`, and `git diff --check`.

Release link-only consumers under ignored `target/b7-link-probe` passed for device and simulator:
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --locked --offline --release --manifest-path target/b7-link-probe/Cargo.toml --target aarch64-apple-ios`
and
`IPHONEOS_DEPLOYMENT_TARGET=14.0 cargo build --locked --offline --release --manifest-path target/b7-link-probe/Cargo.toml --target aarch64-apple-ios-sim`.
`otool -L` reports UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem.B` on both.
Blocks and dispatch symbols (`__Block_copy`, `__Block_release`, `__NSConcreteStackBlock`,
`_dispatch_async_f`) resolve through `libSystem.B`; no separate Blocks framework or Swift runtime
is linked. The device binary records minos 10.0 and the simulator binary minos 14.0. The consumer
was built only; it was not run.

A package-local repeatable gate is now defined at
`sh platform/ios/ios-sharing/check-share-link-imports.sh`, with its source probe at
`platform/ios/ios-sharing/examples/share_link_check.rs`. It isolates the `share` feature, builds
Release probes for device minos 10.0 and simulator minos 14.0, and checks direct imports, UIKit
selector markers, clipboard-feature exclusion, and the recorded minos. The gate passed after root
integration. Both binaries import exactly CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`,
and `libobjc.A.dylib`; selector markers were present, clipboard/Swift/Python symbol checks passed,
and `vtool` confirmed device minos 10.0 and Simulator minos 14.0 with SDK 26.5. The initial run
exposed a case-insensitive Python-symbol pattern that falsely matched `_objc_copyWeak`; the gate
now uses case-sensitive Python C API matching. The probe artifacts were not executed. The gate is
wired in macOS CI, but no passing workflow run is recorded.

The iPhoneOS 26.5 SDK marks `UIActivityViewController` iOS 6.0, its
`completionWithItemsHandler` iOS 8.0, the popover controller and popover style iOS 8.0, and
`UIViewController.present` iOS 5.0. The UIKit API floor is iOS 8.0; the current Rust target
artifacts have higher device/simulator minimums of 10.0/14.0. No device-only restriction was found
in the reviewed public UIKit declarations. The host is Xcode 26.6 (build 17F113) with
iPhoneOS/iPhoneSimulator SDK 26.5, below the required Xcode 27.x baseline. No simulator/device UI,
privacy prompt, recipient behavior, or live delivery was tested or is claimed.

## Objective

Add an iOS `ShareBackend` for `framework-sharing::ShareClient` via public UIKit `UIActivityViewController`

Use caller-supplied UI context and an explicit iPad popover anchor. Add no Swift source, global registry, or executor

## Dependencies

- Foundation A, iOS runtime B, and D6 `PLAN_CAPABILITIES_SHARE.md` are integrated
- B6 `ios-sharing` is integrated; extend its crate with no change to clipboard semantics
- Use current workspace `objc2`, `objc2-foundation`, `objc2-ui-kit`, and `block2` deps where suitable

## Read first

- `PLAN.md`
- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES_SHARE.md`
- `crates/framework-sharing/src/share.rs`
- `platform/ios/ios-sharing/src/lib.rs`
- `platform/ios/ios-runtime/src/main_thread.rs`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- Apple `UIActivityViewController` and popover docs linked below

## Write scope

- `platform/ios/ios-sharing/**`
- `docs/ios/sharing.md`

Do not edit portable share or clipboard contracts, root workspace config, `Cargo.lock`, capability status, global CI, C bindings, or Swift ABI. Report shared dep or lock needs to the orchestrator

## Backend contract

- Add a target-gated iOS backend for `ShareBackend`; keep UIKit and Foundation types out of `framework-sharing`
- Use a caller-supplied presenting `UIViewController` and explicit popover source view plus source rectangle; do not select a fake app UI anchor
- Use `UIActivityViewController` with a non-empty activity item array in request order and no custom `UIActivity` values
- Convert `ShareItem::Text` to an owned native string and `ShareItem::Url` via the public URL path. Report invalid URL text as `InvalidInput`; note native canonicalization and do not promise exact URL preservation
- Set `UIPopoverPresentationController.sourceView` and `sourceRect` on iPad; present modally on compact phone UI. Verify all API floors from the active SDK
- Keep construct, poll, present, callback, and drop on the main thread. Mark backend and futures `!Send`; use the typed `MainThread` proof
- Start native work on first future poll. Map a successful activity callback to `Completed`, callback cancel/dismiss to `Dismissed`, and a native error to `ShareError::Backend` with `ErrorKind::Platform` and its `NSError` code when valid
- Use existing Blocks support for completion. Make callback state exactly-once and inert after future drop; clear waker/result state on drop; avoid a global registry, task runtime, or retain cycle
- A Rust panic must not unwind through UIKit or a Block callback
- Define `availability()` without a claim that a visible or active presenter is always ready. A failed presentation preflight must return a bounded error, not leave a future pending forever
- Provide `IosShareSession` for callback-based consumers without changing the borrowed future contract: retain caller-selected presenter/source view, accept an owned request, allow one active operation, and hold no global state or executor
- `IosShareSession::start` checks main-thread access and request/presentation preflight synchronously; rejected starts do not take callback ownership. Accepted operations deliver at most one terminal callback and detach session state before notifying it
- `IosShareSession::cancel` and drop suppress a pending result callback but do not promise to dismiss UIKit UI. Serialize session create/start/cancel/drop on the main thread; route UIKit completion through a safe main-queue handoff because its public completion documentation does not specify a queue
- Defer user callback execution until after `start` returns, including if UIKit calls its completion inline; do not allow callback reentry into the C API

## Scope, privacy, and limits

- Support text and URL-text only; no files, images, custom activities, recipient data, activity IDs, previews, or delivery claims
- The user may dismiss system UI or choose an activity; future drop suppresses Rust completion but does not promise to dismiss UI already on screen
- Verify whether a permission, `Info.plist` key, entitlement, or device-only limit applies; do not infer empty needs from silence
- No live share-sheet action is required in host checks. Do not claim runtime UI, recipient delivery, parity, or performance from compile/link evidence

## Validation and handoff

- Add deterministic host checks for payload conversion, result map, first-poll start, exactly-once completion, and future drop with isolated state
- Run `cargo fmt --all -- --check`, `cargo test -p ios-sharing`, device and simulator `cargo check`, matching Clippy with `-D warnings`, and `git diff --check` where the SDK is present
- Link small device and simulator consumers and inspect imports; report UIKit/Foundation, ObjC runtime, Blocks, and Swift runtime linkage exactly
- Use `sh platform/ios/ios-sharing/check-share-link-imports.sh` for a repeatable share-only Release link/import/minos gate; build and inspect only, do not execute its probe
- Document API floor, UI context, popover anchor, errors, ownership, cancel limits, privacy/config facts, and compile-only limits in `docs/ios/sharing.md`
- Report changed files, commit SHA, checks, imports, verified API facts, runtime evidence, deviations, and open assumptions. Do not push

## Apple refs

- [UIActivityViewController init](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/init%28activityitems%3Aapplicationactivities%3A%29)
- [UIActivityViewController completion](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/completionwithitemshandler-swift.property)
- [UIViewController present](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29)
- [UIPopoverPresentationController sourceView](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourceview)
- [UIPopoverPresentationController sourceRect](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourcerect)
