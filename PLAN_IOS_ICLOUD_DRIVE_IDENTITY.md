# PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md — Workstream B35: iOS iCloud Drive Identity-Presence Backend

## Objective

Implement the D30 presence-only contract with the public Foundation
`NSFileManager.ubiquityIdentityToken` property. Do not implement CloudKit or iCloud file access.

## Read first

- `PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md`
- `docs/capabilities/icloud-drive-identity.md`
- [Apple `FileManager.ubiquityIdentityToken`](https://developer.apple.com/documentation/foundation/filemanager/ubiquityidentitytoken)
- [Apple Technical Q&A QA1935](https://developer.apple.com/library/archive/qa/qa1935/_index.html)
- Installed Foundation SDK header and `objc2-foundation` 0.3.2 generated bindings/features

## Write scope

- `platform/ios/ios-cloud/**` for this bounded slice only
- `docs/ios/icloud-drive-identity.md`

Do not edit the portable D30 crate, root workspace configuration or lockfile, capability status
JSON/counts, aggregate plans, shared documentation indexes, CI, `tools/xtask`, or other cloud APIs.

## Required implementation

- Read `NSFileManager::defaultManager().ubiquityIdentityToken()` and immediately reduce `Option` to
  presence; expose no native token or token-derived identity.
- Use only generated public bindings with the minimum Foundation feature set (`std`,
  `NSFileManager`, and `NSObject`). Do not add unsafe
  Objective-C calls or Swift source.
- Document the iOS API floor from the installed SDK declaration and the Foundation feature set.
- State the host iCloud capability, iCloud Drive Documents service, signing/provisioning, and
  per-app settings requirements without hardcoding an entitlement payload.
- Require no `Info.plist` key and do not inspect or infer entitlements.
- State all ambiguous nil causes and that the result proves neither container access nor CloudKit
  sign-in; no notification, sync, file, CloudKit, or account-name path is in scope.

## Validation

- Compile and Clippy-check `ios-cloud` for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Confirm no token escape, logging, persistence, formatting, comparison, CloudKit API, or Swift
  source is present.
- Record that target checks are compile/lint evidence only, not configured-host or runtime proof.
- Report files, API/dependency metadata, commands, deviations, and unresolved runtime limits.
