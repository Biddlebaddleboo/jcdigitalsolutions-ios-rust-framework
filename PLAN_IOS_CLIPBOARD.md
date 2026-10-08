# PLAN_IOS_CLIPBOARD.md — Workstream B6: iOS Plain-Text Clipboard Backend

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

Do not edit portable clipboard semantics, root workspace files, `Cargo.lock`, the shared capability manifest, global CI, C bindings, Swift ABI, or share-sheet APIs. The orchestrator owns workspace dependency features, lockfile resolution, capability manifest, and CI integration.

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
- Link minimal device and simulator consumers and inspect imports; confirm UIKit/Foundation only as required, with no Swift runtime or unrelated framework.
- Record exact compile/link evidence separately from any simulator privacy prompt or manual paste check. Do not claim live user-pasteboard behavior, permission UX, parity, or performance without direct evidence.

## Handoff

Report changed files, commit SHA, exact checks, linked imports, verified OS floor and privacy facts, actual runtime evidence, deviations, and unresolved assumptions. Do not push.

## Apple references

- [UIPasteboard](https://developer.apple.com/documentation/uikit/uipasteboard)
- [UIPasteboard.string](https://developer.apple.com/documentation/uikit/uipasteboard/string)
- [UIPasteControl](https://developer.apple.com/documentation/uikit/uipastecontrol)
