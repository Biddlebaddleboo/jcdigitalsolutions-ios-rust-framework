# PLAN_BINDINGS_PREFERENCES.md — F9: iOS Preferences C API

## Status

F9 adds an opt-in `ios-preferences` feature to `framework-c-api` and
`framework_ios_preferences.h` over D1 `framework-preferences` and B1 `ios-preferences`. The API
contains one opaque handle and create, destroy, availability, byte get/set, and remove calls. It
keeps the storage non-secure and capability-scoped; it does not broaden the portable D1 contract.

Implementation paths are `bindings/c/src/ios_preferences.rs`,
`bindings/c/include/framework_ios_preferences.h`,
`bindings/c/abi-manifest.json`, `docs/bindings/ios-preferences.md`, and
`bindings/c/check-ios-preferences.sh`. `docs/DOCUMENTATION.md` links the guide. Persistent macOS CI
invokes that focused check from `.github/workflows/ci.yml`.

## Contract evidence

- `crates/framework-preferences/src/lib.rs` defines exact, case-sensitive UTF-8 keys; keys must be
  non-empty and contain no NUL byte.
- `get_bytes` returns owned bytes or `None`; an empty stored value remains distinct from absence.
- `set_bytes` borrows input for the call. `RequireAtomic` must return `Unsupported` before any
  mutation if the backend cannot promise one-key atomicity. A later read through the same backend
  must see a successful write unless another writer changes the key. No crash-durability or
  multi-key transaction promise exists.
- `remove` returns whether a value was visible before removal. `docs/ios/preferences.md` records
  that `NSUserDefaults` checks its search list, then removes from the app domain; a registered or
  global-domain fallback may appear on a later read.
- `platform/ios/ios-preferences/src/lib.rs` uses a retained
  `NSUserDefaults.standardUserDefaults` object and Foundation `NSData` values. Its write policy
  always reports `NotGuaranteed`; `RequireAtomic` returns `Unsupported` before mutation.
- This is non-secure settings storage, not Keychain, cross-device sync, or an immediate durable
  flush. No permission prompt or entitlement is required. The consuming app must include the
  required `NSUserDefaults` usage reason in `PrivacyInfo.xcprivacy`; this crate does not supply the
  app privacy manifest.
- `platform/ios/ios-preferences/Cargo.toml` pins existing workspace `objc2` and
  `objc2-foundation` dependencies with `NSData`, `NSString`, and `NSUserDefaults` features.
- The locked `objc2-foundation` 0.3.2 binding's `user_defaults.rs` marks `NSUserDefaults`
  `Send + Sync`, consistent with the thread-safety note in `docs/ios/preferences.md`. A future C
  handle can permit serialized calls from any thread; it must prohibit simultaneous calls on one
  handle unless its Rust implementation adds synchronization.

## Implemented C boundary

Keep the first C API iOS-only and expose one opaque `FrameworkIosPreferences` handle with
create/destroy plus byte `get`, `set`, and `remove` calls. Reuse F1 `FrameworkStr`, `FrameworkSlice`,
`FrameworkOwnedBuffer`, and `FrameworkStatus`; do not expose Foundation types or `Retained`.

- `get` must initialize `out_found` and `out_value` before validation, distinguish absent from an
  empty value, and require F1's owned-buffer destructor for present bytes.
- `set` must copy borrowed bytes before return. Its status and output must report the current B1
  `NotGuaranteed` atomicity; if its ABI accepts an atomicity requirement, `RequireAtomic` must fail
  before any native mutation.
- `remove` must state the search-list visibility rule above, not imply that an app-domain value
  existed or that no fallback will reappear.
- The C contract must state that calls on one handle cannot run at the same time. The existing
  `NSUserDefaults` binding supports serialized use from different threads; no main-thread rule is
  needed for this backend.
- Map `PreferenceError::kind()` with `FrameworkStatus::from_error`. The current B1 implementation
  does not produce a platform-native code; add no native-code output unless a concrete backend path
  can supply one.
- Non-iOS calls must validate required outputs and key/value shapes, then return
  `FRAMEWORK_STATUS_UNSUPPORTED` without a fake backend.

## Validation record

Xcode 27 run `38041994806` on head `51959d40530dc4b9e51a255acf6d5f1d30becdc4` passed the
header-only C++17 gate at step 235, then failed the optional preferences C ABI step 241. Its iOS
10.0 C++ compile reached the Xcode 27 libc++ `stddef.h` shim and promoted the SDK warning
`The selected platform is no longer supported by libc++` to an error. The C++ fixture uses C ABI
headers only, so `check-ios-preferences.sh` passes `-nostdinc++` for its C++ compile commands;
this avoids the unsupported libc++ header shim while preserving the C++ runtime link/import
assertion for `libc++.1.dylib`, `-Werror`, and the device/Simulator minos checks. Xcode 27 run
`38043226353` on head `3e2c7e66538df74e88a255de4da22920146c4b05` passed the optional preferences
C ABI compile/link step 241 in job `114187479698`. Hosted output confirmed C11/C++17 import checks
and iOS 10.0 device / 14.0 Simulator minimums; probes were not executed. No consumer executable
was run, and this compile/link pass does not claim preferences runtime behavior

On 2026-10-08, `cargo check --offline -p framework-c-api --no-default-features --features
ios-preferences` refreshed the shared lock and passed. It retained the workspace package
resolution for `ios-media-library-status` and `objc2-media-player` added by the row-090 workstream.

`sh bindings/c/check-ios-preferences.sh` then passed. It checks manifest/header symbols and tags,
the default, enabled-iOS, and enabled-host dependency graphs, Rust host/device/Simulator checks and
strict Clippy, C11/C++17 host compile/link, arm64 device/Simulator release archives, C11/C++17
device/Simulator link probes, direct imports, Objective-C selectors, and deployment metadata. The
script never executes the consumers. All four cross-target consumers imported Foundation,
`libSystem.B.dylib`, and `libobjc.A.dylib`; the two C++ consumers also imported
`libc++.1.dylib`. Expected `NSUserDefaults` class/selector strings were present; no UIKit,
Security, Network, StoreKit, Swift, Python, AppKit, WebKit, or UserNotifications import was found.

`vtool -show-build` reported iOS device `minos` 10.0 and SDK 26.5 (`LC_VERSION_MIN_IPHONEOS`),
and Simulator `minos` 14.0 and SDK 26.5 (`LC_BUILD_VERSION`), for both C11 and C++17 probes. These
are the validation target floors, not an established minimum OS claim for `NSUserDefaults`.
`xcodebuild -version` reported Xcode 26.6, build 17F113. No live defaults behavior or consuming-app
privacy-manifest compliance is validated. No tests were added or run.

## Scope used

- `bindings/c/**` capability files, the focused binding doc, and its CI step
- `PLAN_BINDINGS_PREFERENCES.md`
- No edits to the portable D1 contract, B1 backend, root workspace glob, global capability counts,
  or unrelated C ABI features

No tests were added or run for this workstream. Do not claim live defaults behavior, atomic updates,
durable flush, Keychain secrecy, or cross-device synchronization
