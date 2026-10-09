# PLAN_IOS_SPRITEKIT.md — Workstream B70: iOS SpriteKit Node Position

## Status

B70 adds one main-thread-owned detached `SKNode` facade with create/get/set operations for `position` only. It does not expose a scene, view, renderer, or general SpriteKit API.

## Evidence and API surface

- Apple `SKNode` docs mark the class `@MainActor`, specify parent-coordinate `position`, and state that the base node draws no content
- iOS 26.5 `SpriteKit/SKNode.h` declares `-init`, `position` (`CGPoint`), and `setPosition:` without per-symbol availability annotations; Apple's iOS 6.1-to-7.0 API diff lists SpriteKit as a framework added in iOS 7.0, the API floor used for these unannotated members
- With `TARGET_OS_IPHONE` and `SKVIEW_AVAILABLE`, the SDK declares iOS `SKNode : UIResponder`; the macOS branch instead inherits `NSResponder`
- No cached `objc2-sprite-kit` 0.3.2 binding exists; a typed `objc2::extern_class!` declaration and the three audited selector declarations are local to this crate

## Ownership and limits

- Constructor requires `MainThread`; query/update recheck main-thread context
- The retained node handle is `!Send`/`!Sync`, preventing migration and off-main `Drop`
- `native_node` returns a borrow tied to the wrapper; any scene, view, hierarchy placement, and rendering remain host-owned
- No `Info.plist` usage description, privacy permission, entitlement, task loop, file access, or Swift source is used
- Build/link evidence does not prove scene integration, visual output, runtime parity, performance, animation, or physics

## Validation

- Run `platform/ios/ios-spritekit/check.sh` for host/device/Simulator check, strict Clippy, rustdoc, formatting, and no-test gates
- Run `platform/ios/ios-spritekit/check-link-imports.sh`; build-only probes must not execute
- Run shell syntax and `git diff --check`
- Record SDK API floor separately from the SDK-specific device/Simulator minimum deployment targets

## Validation record

On Rust 1.94.1, Xcode 26.6 build 17F113, and iOS device/Simulator SDK 26.5, the host/device/Simulator checks, strict Clippy gates, rustdoc, formatting, and release link/import checks pass in `/private/tmp/jc-ios-spritekit-worktree`. Link probes were built but never executed; no tests were run.

The public API floor is iOS 7.0. SDK `SDKSettings.plist` minimum deployment is iOS 12.0; the device probe targets 12.0, while the Simulator probe targets the repository baseline 14.0. These are toolchain validation minimums, not a higher API requirement.

- `sh platform/ios/ios-spritekit/check.sh`
- `sh platform/ios/ios-spritekit/check-link-imports.sh`
- `sh -n platform/ios/ios-spritekit/check.sh platform/ios/ios-spritekit/check-link-imports.sh`
- `git diff --check` for tracked and new workstream files
