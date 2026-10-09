# PLAN_IOS_SECURE_STORAGE.md — Workstream B2: iOS Keychain Backend

## Status

B2 is integrated as a caller-owned `ios-secure-storage` backend over public Keychain Services
APIs. Locked device and simulator checks plus strict all-target Clippy pass. The current local CI
workflow preserves those target gates. The opt-in C link probe calls all three storage APIs on
device and simulator; both probe imports are `CoreFoundation`, `Security`, and `libSystem.B.dylib`.
This probe-scoped evidence does not establish imports for arbitrary host apps. Host policy/status
tests exist but were not run in this execution; no live Keychain or signed-app behavior is claimed

The recorded target-check host is Xcode 26.6 with iPhoneOS/iPhoneSimulator SDK 26.5, below the
required Xcode 27.x baseline.

Access-group behavior is not default-only: `SecItemAdd` omits `kSecAttrAccessGroup` and creates in
the app's default group, while unfiltered `SecItemCopyMatching`, `SecItemUpdate`, and
`SecItemDelete` search all access groups available to the app. The backend exposes no explicit
group-selection API; an app with multiple access-group entitlements can therefore read, update, or
delete a matching service/account item in another entitled group. Strict default-group isolation
remains an open design item. See the [iOS secure-storage guide](docs/ios/secure-storage.md).

## Objective

Implement the iOS backend for D2 `framework-secure-storage` with public Keychain Services APIs. Keep the portable contract `no_std` and map only the contract's device-unlock and device-bound policy flags.

## Dependencies

- Foundation A and iOS runtime B are integrated
- D2 secure-storage contract is integrated
- Installed iOS SDK metadata is inspected before stating availability

## Write scope

- `platform/ios/ios-secure-storage/**`
- `docs/ios/secure-storage.md`

Do not edit the portable D2 crate, root workspace configuration, shared capability manifest, files/preferences/network backends, Swift ABI, C ABI, or other capability families. `platform/ios/*` is already a workspace glob.

## Required implementation

- Implement `SecureStorageBackend` with caller-owned, stateless backend value and no global service registration
- Use public Security/Keychain Services C APIs for generic-password item lookup, add/update, read, and delete
- Use the smallest supported Rust binding surface for Security and CoreFoundation; document why the dependency is required and expose no dependency types in the portable API
- Map `AccessPolicy` to a public Keychain accessibility class; reject any policy the adapter cannot meet before mutation and return the effective policy
- Preserve `OSStatus` as the native code; map item-not-found to `None`/`false` only for read/remove
- Copy returned secret bytes into caller-owned storage and document plaintext exposure, synchronous call cost, and native error behavior
- Do not claim biometric/per-access user authentication, shared access groups, iCloud Keychain sync, secure deletion, zeroization, crash durability, or hardware-backed custom-key support
- Do not add `.swift` source, custom cryptography, private APIs, or a runtime/global registry

## Validation and handoff

- Run `cargo check` and Clippy for the backend on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add deterministic tests for policy mapping and native-status translation when they can run without a live app Keychain
- Link a C probe that calls read, store, and remove on iOS device and simulator; use `otool -L` to enforce the exact direct import set `CoreFoundation`, `Security`, and `libSystem.B.dylib`, and reject Swift, Objective-C, network, and unrelated framework imports for that probe
- Document minimum iOS version only from installed SDK metadata, data-copy cost, callback/thread behavior, native escape handles, and actual entitlements/Info.plist requirements
- Report exact checks, changed files, commit SHA, deviations, and unavailable live-device checks
