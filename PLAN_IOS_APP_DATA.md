# PLAN_IOS_APP_DATA.md — Workstream B1: iOS Files and Preferences

## Status

B1's file and preferences backends are in the tree. Both library roots are gated by
`#![cfg(target_os = "ios")]`; `IosFiles::new` resolves Foundation app-sandbox URLs, while
`IosPreferences::new` accesses `NSUserDefaults.standardUserDefaults`. Host-runnable tests reuse the
production `path_parts` and preference write-policy helpers. They do not instantiate the gated
backends or exercise descriptor-relative I/O, symlinks, atomic rename, live sandbox roots,
`NSUserDefaults`, or persistence. The native path gate rejects Windows drive prefixes.

Host app-data tests passed: `cargo test --locked --offline -p ios-files -p ios-preferences` (four
integration tests). Portable contract tests passed: `cargo test --locked --offline -p
framework-files -p framework-preferences` (eight tests). Device and simulator checks passed:
`cargo check --locked --offline -p ios-files -p ios-preferences --target aarch64-apple-ios` and
`cargo check --locked --offline -p ios-files -p ios-preferences --target aarch64-apple-ios-sim`.
Strict Clippy passed:
`cargo clippy --locked --offline --all-targets -p ios-files -p ios-preferences --target aarch64-apple-ios -- -D warnings`
and
`cargo clippy --locked --offline --all-targets -p ios-files -p ios-preferences --target aarch64-apple-ios-sim -- -D warnings`.
`cargo fmt --package ios-files --package ios-preferences -- --check` and `git diff --check` passed.

An isolated minimal consumer source and manifest were generated under ignored
`target/b1-link-probe`. Device and simulator builds passed with
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b1-link-probe/Cargo.toml --release --target aarch64-apple-ios` and
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b1-link-probe/Cargo.toml --release --target aarch64-apple-ios-sim`.
`otool -L target/b1-link-probe/target/aarch64-apple-ios/release/b1-link-probe` and
`otool -L target/b1-link-probe/target/aarch64-apple-ios-sim/release/b1-link-probe` showed
Foundation, CoreFoundation, `libobjc.A`, `libSystem.B`, and `libiconv.2`; neither imports UIKit,
Network, Swift, Python, CoreLocation, UserNotifications, or Security. Device load metadata records
minos 10.0; simulator metadata records minos 14.0. The focused gate
`sh platform/ios/ios-files/check-app-data-link-imports.sh` now builds a link-only example for each
crate and target, checks exact imports, rejects unrelated framework symbols, and checks device
minos 10.0 and simulator minos 14.0. CI runs this gate after both crates' target checks and strict
all-target Clippy. The probe executables were inspected but not run. Checks ran with Xcode 26.6 and
iPhoneOS/iPhoneSimulator SDK 26.5, below the required Xcode 27.x baseline.

The base `Files` facade does not accept user-selected, security-scoped, iCloud, or provider URLs. The
separate `IosFileCoordinator` can coordinate caller-supplied URLs synchronously, but does not manage
security scope, prove sandbox containment, or provide file-provider lifecycle support. The separate
`IosSecurityScopedAccess` guard balances one already-issued scope and does not change the base
facade; see [PLAN_IOS_SECURITY_SCOPED_ACCESS.md](PLAN_IOS_SECURITY_SCOPED_ACCESS.md). Preference
durability remains unspecified; `NSUserDefaults` writes are visible in-process before asynchronous
persistence.
Descriptor-relative no-follow traversal does not serialize concurrent native directory renames;
another handle can rename an opened directory inode outside the selected root while its descriptor
continues to refer to that inode, so selected-root containment is not guaranteed under that race.
B14 adds `IosFiles::adopt_url_session_download` as a row-010 sandbox-file operation, distinct from
B17 file coordination; it synchronously adopts only the URLSession callback temporary file into an
`AppPath`. Its copy, commit, and inherited directory-rename limits are recorded in
`PLAN_IOS_FILE_ADOPTION.md`.

The B83 app-data follow-up `IosResolvedBookmark::resolve_unscoped` maps caller-owned, non-security-scoped
Foundation bookmark data to a file URL and stale bit. The method is unsafe because its input scope
cannot be verified; `NSURLBookmarkResolutionWithoutImplicitStartAccessing` does not govern
security-scoped bookmark data. It needs iOS 14.2 and does not create bookmarks, prompt, grant access,
coordinate I/O, or extend sandbox containment. See
[`PLAN_IOS_BOOKMARK_RESOLUTION.md`](PLAN_IOS_BOOKMARK_RESOLUTION.md). B83 is distinct from B82,
which adds only a registered-domain count for the calling app's own FileProvider extension; see the
[B82 FileProvider count plan](PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md).

## Objective

Implement the iOS backends for the D1 `framework-files` and `framework-preferences` contracts using public iOS filesystem/Foundation APIs. Keep the portable crates `no_std`; platform code may use the platform runtime but must not change portable semantics.

## Dependencies

- Foundation A is integrated
- iOS runtime B is integrated
- D1 `framework-files` and `framework-preferences` contracts are integrated

## Write scope

- D1's sandbox implementation in `platform/ios/ios-files/src/lib.rs` and its existing helpers; B14 owns the URLSession adoption operation and B17 owns the file-coordination module, while root reconciles shared exports and indexes
- `platform/ios/ios-preferences/**`
- `docs/ios/files.md`
- `docs/ios/preferences.md`
- focused iOS backend tests within these crates

Do not edit D1 crates, root workspace configuration, Swift ABI, C bindings, network backend, or unrelated capability families. `platform/ios/*` is already a workspace glob.

## Required implementation

- Implement the D1 `FileBackend` contract with caller-owned backend state and no global service registration.
- Resolve each `AppDirectory` through a documented public iOS sandbox API; keep user-selected, security-scoped document-provider URLs out of this base sandbox backend.
- Enforce the portable relative-path contract at the native boundary. Document symlink handling and concurrent directory-rename limits separately; do not claim selected-root containment where descriptor-relative operations can outlive a native rename of a traversed directory.
- Implement create/replace modes distinctly. If atomic replacement is required, use a same-directory temporary file and atomic rename where the selected API guarantees it; do not equate atomic visibility with crash durability.
- `ReplaceExisting` must require an existing regular file: map a missing target to `NotFound` and final symlink, directory, or special entries to `InvalidInput`. Check before staging and immediately before atomic `RENAME_SWAP`; document that these checks cannot serialize same-path native mutation, including in the non-atomic open/truncate/write fallback.
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
