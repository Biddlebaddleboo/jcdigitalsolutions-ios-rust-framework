# PLAN_IOS_APP_DATA.md — Workstream B1: iOS Files and Preferences

## Objective

Implement the iOS backends for the D1 `framework-files` and `framework-preferences` contracts using public iOS filesystem/Foundation APIs. Keep the portable crates `no_std`; platform code may use the platform runtime but must not change portable semantics.

## Dependencies

- Foundation A is integrated
- iOS runtime B is integrated
- D1 `framework-files` and `framework-preferences` contracts are integrated

## Write scope

- `platform/ios/ios-files/**`
- `platform/ios/ios-preferences/**`
- `docs/ios/files.md`
- `docs/ios/preferences.md`
- focused iOS backend tests within these crates

Do not edit D1 crates, root workspace configuration, Swift ABI, C bindings, network backend, or unrelated capability families. `platform/ios/*` is already a workspace glob.

## Required implementation

- Implement the D1 `FileBackend` contract with caller-owned backend state and no global service registration.
- Resolve each `AppDirectory` through a documented public iOS sandbox API; keep user-selected, security-scoped document-provider URLs out of this base sandbox backend.
- Enforce the portable relative-path contract at the native boundary. Document symlink handling and do not claim path containment if a backend operation can escape the selected root.
- Implement create/replace modes distinctly. If atomic replacement is required, use a same-directory temporary file and atomic rename where the selected API guarantees it; do not equate atomic visibility with crash durability.
- Implement D1 preference bytes through `NSUserDefaults`-compatible property-list storage without claiming Keychain secrecy, cross-device sync, or immediate durable flush.
- Report `NotGuaranteed` preference atomicity and reject an atomicity request only when the D1 contract requires that behavior.
- Preserve stable framework error categories and the relevant POSIX/Foundation native code.
- Record Foundation/Objective-C linkage only in the iOS crates that need it. Reuse the existing locked Objective-C/Foundation crate versions and minimal features; add no unrelated dependency.

## Validation and handoff

- Run `cargo check` and Clippy for both crates on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run portable contract tests from the integrated D1 crates; add deterministic backend tests only for semantics this adapter can prove without user permission or a live app sandbox.
- Inspect both target binaries or a minimal consumer for imported frameworks; confirm network, Swift, and unrelated capability frameworks are absent.
- Document minimum iOS version only when verified from SDK metadata, required Info.plist keys, permission/entitlement state, path and value-copy costs, callback/thread behavior, and native escape handles.
- Report exact checks, unsupported provider-aware file coordination, user-selected document access, preference durability, and any host/toolchain caveat.
