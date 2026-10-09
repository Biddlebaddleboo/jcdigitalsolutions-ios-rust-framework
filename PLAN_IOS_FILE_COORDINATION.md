# PLAN_IOS_FILE_COORDINATION.md — Workstream B17: Foundation File Coordination

## Status

The installed `objc2-foundation` 0.3.2 bindings expose both single-URL read and write
`NSFileCoordinator` accessors with safe generated signatures. B17 adds `IosFileCoordinator` with
synchronous callback scopes and native Foundation error-code preservation. Locked device and
simulator checks plus strict all-target Clippy pass on Xcode 26.6 / SDK 26.5, below the required
Xcode 27.x baseline. These checks are compile/lint evidence only; no tests or live coordinated
access ran. No FileProvider, document-picker, or security-scope behavior is claimed

## Objective

Add an iOS-only Foundation extension for coordinated read and write access to caller-supplied file
URLs. Keep the existing sandbox-relative D1 `IosFiles` contract unchanged

## Dependencies

- Existing iOS `ios-files` backend and `framework-files` error contract
- `ios-runtime` for panic containment at the Objective-C block boundary
- Workspace-pinned `block2` and `objc2-foundation`; enable only `NSFileCoordinator`, `NSError`,
  `NSURL`, and block support required by the generated API

## Write scope

- `PLAN_IOS_FILE_COORDINATION.md`
- `platform/ios/ios-files/Cargo.toml`
- `platform/ios/ios-files/src/lib.rs`
- `platform/ios/ios-files/src/coordination.rs`
- `docs/ios/file-coordination.md`

Do not edit `framework-files`/D1, root `Cargo.toml`, `Cargo.lock`, capability plan/index/README,
CI, or unrelated platform crates. Root owns Cargo and shared-doc integration

## Required contract

- Expose a caller-owned `IosFileCoordinator` that creates one `NSFileCoordinator` without a
  registered file presenter and offers `coordinate_read` and `coordinate_write` for one file URL
- Use public generated bindings for the single-item `coordinateReadingItemAtURL...byAccessor:`
  and `coordinateWritingItemAtURL...byAccessor:` methods. The installed binding provides both
  methods, so this slice need not narrow to only one direction
- Take typed Foundation read/write option values from the caller. Reject a URL that is not a file
  URL as `InvalidInput`; do not normalize or rewrite the supplied URL
- Pass the coordinated URL from Foundation to an `FnOnce` accessor only for that synchronous
  accessor call. Do not retain it or claim the original URL is the URL to use inside the accessor
- Return the accessor result/error unless Foundation reports an `NSError`, which takes precedence.
  Catch an accessor panic before it can unwind through the native block, map it to `Internal`, and
  document panic-hook and `panic=abort` behavior
- Keep the coordinator `!Send`/`!Sync`; methods borrow it mutably, block the calling thread, and
  use no executor. Document no-main-thread requirement, same-URL reentrancy/deadlock risks, and
  that coordination failures may skip the accessor
- Map Foundation `NSError` to `FileError::Backend(ErrorKind::Platform)` while preserving its code
  when it fits `PlatformErrorCode`'s `i32`; the portable error has no Foundation domain/userInfo
  field. Preserve an accessor's own `FileError`
- Do not change D1 path confinement, file read/write implementation, or claims about sandbox roots.
  This URL extension does not establish root containment or security-scoped access
- Do not claim FileProvider/document-picker, multi-URL coordination, presenter registration,
  move/delete operations, cancellation, timeout, async access, cross-process integration, or
  runtime validation

## Validation and handoff

- No tests, Cargo commands, live file operations, provider calls, or document-picker calls in this
  isolated slice
- Run direct formatting, source/manifest review, and diff checks only. Root runs Cargo, device, and
  simulator gates after integrating dependencies and lockfile
- Report exact objc2 binding signatures and API scope, direct checks, error mapping, deviations,
  and remaining target/runtime evidence
