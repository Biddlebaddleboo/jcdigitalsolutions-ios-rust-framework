# PLAN_IOS_SHARE.md — Workstream B7: iOS Outgoing Share UI Backend

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

## Scope, privacy, and limits

- Support text and URL-text only; no files, images, custom activities, recipient data, activity IDs, previews, or delivery claims
- The user may dismiss system UI or choose an activity; future drop suppresses Rust completion but does not promise to dismiss UI already on screen
- Verify whether a permission, `Info.plist` key, entitlement, or device-only limit applies; do not infer empty needs from silence
- No live share-sheet action is required in host checks. Do not claim runtime UI, recipient delivery, parity, or performance from compile/link evidence

## Validation and handoff

- Add deterministic host checks for payload conversion, result map, first-poll start, exactly-once completion, and future drop with isolated state
- Run `cargo fmt --all -- --check`, `cargo test -p ios-sharing`, device and simulator `cargo check`, matching Clippy with `-D warnings`, and `git diff --check` where the SDK is present
- Link small device and simulator consumers and inspect imports; report UIKit/Foundation, ObjC runtime, Blocks, and Swift runtime linkage exactly
- Document API floor, UI context, popover anchor, errors, ownership, cancel limits, privacy/config facts, and compile-only limits in `docs/ios/sharing.md`
- Report changed files, commit SHA, checks, imports, verified API facts, runtime evidence, deviations, and open assumptions. Do not push

## Apple refs

- [UIActivityViewController init](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/init%28activityitems%3Aapplicationactivities%3A%29)
- [UIActivityViewController completion](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/completionwithitemshandler-swift.property)
- [UIViewController present](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29)
- [UIPopoverPresentationController sourceView](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourceview)
- [UIPopoverPresentationController sourceRect](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourcerect)
