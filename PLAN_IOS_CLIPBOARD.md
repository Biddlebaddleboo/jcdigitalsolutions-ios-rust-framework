# PLAN_IOS_CLIPBOARD.md — Workstream B6: iOS Plain-Text Clipboard Backend

## Status

B6's UIKit clipboard backend and deterministic host seam are implemented. The backend retains a
typed `MainThread` proof, uses `PhantomData<Rc<()>>` to remain `!Send`, and performs read, write, and
clear only on first future poll. Read checks `hasStrings`, copies the first `strings` value into an
owned Rust `String`, returns `None` only for no string type, and maps conversion or native races to
`ClipboardError::Backend`. Write assigns `string`; clear assigns an empty `items` array, replacing
all pasteboard items. `availability()` is `Unknown`: UIKit exposes no query for current
programmatic-read usability or approval, and `Unknown` does not mean `RequiresPermission`. The B6
backend has no callback, queue, executor, or worker-thread path.

The B84 addition `IosClipboardBackend::has_plain_text()` is a synchronous, caller-invoked presence
snapshot. It reads only `UIPasteboard.hasStrings`, does not load string content, returns a bool
rather than an item count, and does not change the portable D5 contract or the `Availability`
meaning. Apple lists `hasStrings` among type-checking APIs that avoid user notifications and alerts
when the system has not established user intent; this does not authorize or guarantee a subsequent
content read. The installed iOS SDK marks this selector available from iOS 10.0, matching B6's
existing minimum API floor. B84-specific device and Simulator compile, strict Clippy, and rustdoc
checks passed with the commands recorded in `docs/ios/sharing.md`. The focused Release link/import
gate also passed: device and Simulator imports were exactly UIKit, Foundation, `libobjc.A.dylib`,
and `libSystem.B.dylib`; its selector markers include `hasStrings`; device minos is 10.0 and
Simulator minos is 14.0, both with SDK 26.5. The gate builds and inspects its link-only example but
does not execute it. No tests or live pasteboard runtime checks were run for B84.

Deterministic host tests passed with
`cargo test --locked --offline -p ios-sharing --no-default-features --features clipboard` (four
clipboard tests). The full crate tests passed with `cargo test --locked --offline -p ios-sharing`
(fourteen tests across the clipboard and separate share module, including two later B7 callback
completion-state tests). The clipboard tests cover UTF-8
conversion, backend-error propagation, unpolled drop, first-poll start, and single execution; they
never access the developer's pasteboard. `cargo fmt --all -- --check` and `git diff --check` pass.

Default crate device and simulator checks passed with
`cargo check --locked --offline -p ios-sharing --target aarch64-apple-ios` and
`cargo check --locked --offline -p ios-sharing --target aarch64-apple-ios-sim`. Strict Clippy passed
with `cargo clippy --locked --offline --all-targets -p ios-sharing --target aarch64-apple-ios -- -D warnings`
and
`cargo clippy --locked --offline --all-targets -p ios-sharing --target aarch64-apple-ios-sim -- -D warnings`.
The isolated clipboard-only feature also passed device and simulator checks with
`cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios` and
`cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios-sim`; its strict Clippy checks also passed on both targets.

A temporary link-only consumer under ignored `target/b6-link-probe` used
`default-features = false, features = ["clipboard"]`. Device and simulator Release builds passed
with `IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b6-link-probe/Cargo.toml --release --target aarch64-apple-ios` and
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b6-link-probe/Cargo.toml --release --target aarch64-apple-ios-sim`. `otool -L` on each binary showed UIKit, Foundation, `libobjc.A`, and `libSystem.B`, with no Swift runtime or unrelated framework. The device binary records minos 10.0; the simulator records minos 14.0. Binary strings include `UIPasteboard`, `hasStrings`, `setString:`, and `setItems:`; no `UIActivityViewController` or `UIActivity` symbol appears in this clipboard-only consumer. These binaries were not run.

The Xcode 26.6 / iPhoneOS and iPhoneSimulator SDK 26.5 headers mark `UIPasteboard` and its `items`
property available from iOS 3.0 and `hasStrings` from iOS 10.0; B6's effective floor is iOS 10.0.
Apple documents a notice from iOS 14 when another app's general-pasteboard content is read without
user intent, and an approval alert for programmatic pasting in iOS 16 and later; `UIPasteControl`
is out of scope. Public headers and reviewed Apple docs specify no usage-description key or
entitlement for this path; that is not a query for, or guarantee about, OS privacy UI. No device or
simulator clipboard action, privacy prompt, or live pasteboard behavior was tested. The host Xcode
26.6 / SDK 26.5 is below the planned Xcode 27.x baseline.

The B6-only Release link/import/minos gate `sh platform/ios/ios-sharing/check-clipboard-link-imports.sh`
is wired into macOS CI; it disables default features and selects only `clipboard`, then links a
Release probe for device minos 10.0 and
simulator minos 14.0. It checks exact direct imports `Foundation`, `UIKit`, `libSystem.B.dylib`, and
`libobjc.A.dylib`, plus `UIPasteboard`, `generalPasteboard`, `hasStrings`, `strings`, `firstObject`,
`dataUsingEncoding:`, `setString:`, and `setItems:` markers; it rejects
share-only and Swift/Python symbols and verifies minos with `vtool`. The gate builds and inspects but
does not execute the probe. The updated gate passed in the integrated checkout; direct imports are
exactly Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, with minos 10.0 on device and
14.0 on Simulator. It asserts `generalPasteboard`, `strings`, `firstObject`, and
`dataUsingEncoding:` along with the type-preflight/write/clear markers. The CI step is present, but
no passing workflow run is recorded.

The package default feature set also includes the separate B7 `share` backend; both the B6 gate and
F6 C ABI gate disable default features and select `clipboard`. No live pasteboard action, privacy
prompt, or user-facing clipboard behavior was tested. G113 records this compile/link/import gate;
the probes were not executed. The B6 executor made no standalone commit per task scope; its source,
guide, and gate are integrated in checkpoint `07cd525`.

## Objective

Implement `framework-sharing::ClipboardBackend` on iOS with public UIKit `UIPasteboard` APIs, no Swift source, no global registry, and no mandatory executor.

## Dependencies

- Foundation A, iOS runtime B, and D5 `PLAN_CAPABILITIES_CLIPBOARD.md` are integrated.
- `framework-sharing` is in the workspace and the portable contract is stable.
- Use the existing `objc2-ui-kit` workspace dependency; report any feature or shared-dependency need to the orchestrator.

## Read first

- `PLAN.md`
- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES_CLIPBOARD.md`
- `crates/framework-sharing/src/lib.rs`
- `platform/ios/ios-runtime/src/main_thread.rs`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- Apple `UIPasteboard`, `UIPasteboard.string`, and `UIPasteControl` documentation linked below

## Write scope

- `platform/ios/ios-sharing/**`
- `docs/ios/sharing.md`
- capability-specific tests owned by `ios-sharing`
- `platform/ios/ios-sharing/examples/clipboard_link_check.rs` and `platform/ios/ios-sharing/check-clipboard-link-imports.sh`

Do not edit portable clipboard semantics, root workspace files, `Cargo.lock`, the shared capability manifest, C bindings, Swift ABI, or share-sheet APIs. The orchestrator owns workspace dependency features, lockfile resolution, capability manifest, and CI integration; it may wire this package-local validation gate into CI as a separate root integration step.

## Backend requirements

- Add an iOS-only `ios-sharing` crate and implement the portable `ClipboardBackend` with compile-time selection and public `UIPasteboard.generalPasteboard` APIs.
- Keep Apple types in the iOS backend. Do not add `UIPasteboard`, `NSString`, `MainThreadMarker`, or UIKit types to `framework-sharing`.
- Use the integrated typed main-thread proof. Construction and every native access must obey UIKit thread constraints; document whether the future is `!Send` and ensure it cannot move access to a worker thread.
- Operations start on first future poll. This backend's UIKit calls are synchronous on the required thread; do not add a queue, executor, task registry, or callback block. Dropping an unpolled operation has no pasteboard effect.
- Read the first plain-text value as an owned Rust `String`. Check `hasStrings` before loading the string. Return `Ok(None)` only when no readable string exists; map conversion/native failure to `ClipboardError::Backend`, not `None`.
- Write UTF-8 text through the UIKit general pasteboard. Document Foundation/UIKit conversion copies and native behavior that replaces or alters other item representations.
- Implement clear with a public API that reliably removes the plain-text value. Verify whether the selected UIKit operation also removes other pasteboard items; document this native effect because D5 does not promise preservation or removal of rich representations.
- Preserve `Availability` and `ErrorKind` semantics without exposing UIKit errors in portable types. Do not claim a permission query or stable permission state if UIKit exposes none.
- No callback may panic across an Objective-C/C boundary. Prefer no callback path for this synchronous backend.
- No share sheet, `UIPasteControl`, named pasteboards, images/files/rich representations, background access, or clipboard history API in this slice.

## Privacy and availability evidence

- Document that reading another app's general-pasteboard content without user intent may show a system notice on iOS 14 and later, and programmatic reads may show a user approval alert on iOS 16 and later.
- `UIPasteControl` can provide a user-initiated paste path without the programmatic-read prompt, but it is UI integration and is explicitly out of scope here. Do not claim this backend avoids prompts.
- Verify the API availability floor, UIKit linkage, simulator/device target checks, and any Info.plist or entitlement requirements from the active SDK and Apple documentation. Do not infer empty requirements from an absent permission prompt.
- Do not access or mutate the developer's live general pasteboard in host tests. Keep any simulator manual check explicit and report it separately from compile/test evidence.

## Validation and handoff

- Add deterministic host tests for conversion and operation/future behavior with a fake or isolated seam; do not rely on the live system pasteboard.
- Run `cargo fmt --all -- --check`, `cargo test -p ios-sharing`, `cargo check --locked -p ios-sharing --target aarch64-apple-ios`, the matching simulator check, Clippy with `-D warnings` for both targets, and `git diff --check` where the SDK is available.
- Run `sh platform/ios/ios-sharing/check-clipboard-link-imports.sh` to build and inspect the clipboard-only Release probes, direct imports, selectors, and device/simulator minos; do not execute the probes.
- Record exact compile/link evidence separately from any simulator privacy prompt or manual paste check. Do not claim live user-pasteboard behavior, permission UX, parity, or performance without direct evidence.

## Handoff

Report changed files, commit SHA, exact checks, linked imports, verified OS floor and privacy facts, actual runtime evidence, deviations, and unresolved assumptions. Do not push.

## Apple references

- [UIPasteboard](https://developer.apple.com/documentation/uikit/uipasteboard)
- [UIPasteboard.string](https://developer.apple.com/documentation/uikit/uipasteboard/string)
- [UIPasteControl](https://developer.apple.com/documentation/uikit/uipastecontrol)
