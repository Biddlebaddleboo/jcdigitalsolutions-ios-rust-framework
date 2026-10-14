# PLAN_IOS_DOCUMENT_IMPORT.md — B603: user-selected document open

## Goal

Add an opt-in, native UIKit document-open session to `ios-sharing`, complementing B533's
bytes-backed document export and `ios-files::IosSecurityScopedAccess` / `IosFileCoordinator`.
The session returns the user-selected original security-scoped `NSURL`; it does not read, copy,
persist, or interpret document data.

## Verified baseline

- Framework `main` starts at `1b23785758523efb6de3b2433c18842e33f4a039`.
- Rideshare Bot `origin/main` is `a1858332d36d9796e9f63e753171879f24a9e1be`. The latest tip only changes the app build number; `d7f9e3c` moved peer-recovery policy into Rust. Current non-SwiftUI service calls have framework coverage. Remaining `SwiftUI` view/state composition is app-owned and is not a request to clone SwiftUI.
- `objc2-ui-kit 0.3.2` exposes `UIDocumentPickerViewController::initForOpeningContentTypes_asCopy`; the iOS 26.5 public UIKit header marks it iOS 14.0. `allowsMultipleSelection` is public from iOS 11.0.
- The modern initializer consumes `NSArray<UTType *>`; use the generated `objc2-uniform-type-identifiers 0.3.2` binding. The feature stays opt-in and does not widen `ios-sharing` defaults.
- Apple documents the open-in-place result as a security-scoped URL. The caller starts `IosSecurityScopedAccess` and may use `IosFileCoordinator` for provider-backed reads. Scope start/stop and coordinated I/O remain caller-owned.

## API and safety contract

- Add `document-import` as an opt-in feature on `ios-sharing`; do not change default clipboard/share features.
- Add `IosDocumentImportSession<'ctx>::new(MainThread, &'ctx UIViewController)` and one active
  `start(&mut self, &[&str], FnOnce(DocumentImportResult))` operation. Convert exact UTI identifiers
  to generated `UTType` values inside the opt-in iOS feature so callers need not link or construct
  an iOS 14 type before the runtime-availability check.
- Reject an empty type list, active operation, known-invalid presenter state, missing main-thread
  proof, and OS versions earlier than iOS 14 before UIKit mutation; return the exact uncalled
  callback with a bounded typed start error.
- Construct the picker with `asCopy: false` and `allowsMultipleSelection: false`. Return one
  retained selected `NSURL`, cancellation, or a bounded unexpected-selection failure. Do not claim
  that provider content is readable or durable merely because a URL arrived.
- Keep session, picker, delegate, and callback `!Send`/`!Sync` and main-thread-bound. Exactly-once
  completion detaches state before callback invocation and catches panic at the Objective-C
  boundary. Cancel/drop detaches the callback and delegate but does not dismiss UIKit's presented
  picker or claim file-operation cancellation.
- Preserve the selected URL only by caller ownership. Never start or retain a security scope on the
  caller's behalf. Do not add a Swift file, storage dependency, app-specific type, global registry,
  executor, or permission prompt.

## Write surface

- `platform/ios/ios-sharing/Cargo.toml`
- `platform/ios/ios-sharing/src/lib.rs`
- new `document_import.rs` and host-testable `document_import_state.rs`
- package-local document-import link probe and the existing registered sharing link script
- `docs/ios/sharing.md`, `docs/capabilities/capability-status.json`, `PLAN.md`, and this plan

Do not edit Rideshare Bot, `tools/validation/specs/validation-v1.json`, shared tooling, workspace
defaults, or unrelated capability plans. The existing `ios-sharing` profile already declares the
package source tree, sharing docs, and `check-clipboard-link-imports.sh`; add import-feature host,
device, Simulator, and link checks to that registered script. The spec is 262,072 bytes and the
validator's JSON edit limit is 262,144 bytes, so no new declaration is needed while all added gates
run through this already-registered script.

## Validation and evidence

- Add deterministic tests for empty/valid type policy, first accepted operation, duplicate/late
  completion suppression, cancel/drop detachment, invalid presenter/active-state rejection, and
  returned callback preservation. No sleep or network/provider dependency.
- Run the installed `ios-rust-validate --workspace-root "$PWD" --spec
  "$PWD/tools/validation/specs/validation-v1.json" --capability ios-sharing` after extending its
  registered script.
- Compile and strictly lint the opt-in feature for device and Simulator; link Release probes at
  iOS 14.0 for both targets and verify UIKit, UniformTypeIdentifiers, Foundation, Objective-C, and
  system-library imports. Keep the existing `ios-sharing` device iOS 10.0 / Simulator iOS 14.0
  profile floors for its existing features.
- Run package format/tests, `cargo +1.94.1 xtask docs-check`, report-generation check, and
  `git diff --check`. Do not claim a document-provider UI, file read, security-scope access,
  Simulator execution, or device behavior without runtime evidence.

## Complete when

The optional feature builds and passes its registered host/device/Simulator/link gates, callers can
receive a retained original document URL and use the existing security-scope/coordination APIs, the
capability row and guide describe the exact limits, and this workstream is committed to `main`.
