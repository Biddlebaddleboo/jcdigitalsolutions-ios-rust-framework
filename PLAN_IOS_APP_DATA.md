# PLAN_IOS_APP_DATA.md — active B1 iOS files and preferences

## Evidence / source
The historical bounded B1 filesystem and preferences record is retrievable from Git at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_APP_DATA.md`. Code: `platform/ios/ios-files/src/lib.rs::IosFiles`, `platform/ios/ios-preferences/**`, portable `crates/framework-files/**`, `crates/framework-preferences/**`; canonical semantics in `docs/ios/files.md`. Closed read-only attributes/no-go findings are indexed by `PLAN_IOS_FILES_COMPLETED_AND_DECISIONS.md`. B1 remains partial.

## Required contract and implementation scope
Preserve `AppPath` input validation, caller-owned semantic root descriptors, descriptor-relative traversal, symlink-race defenses and explicit `AT_SYMLINK_NOFOLLOW` semantics. Correct checked native offsets/lengths/overflow, file-kind rules, errors, stable Rust ownership, no unexpected data reads. File-write modes must distinguish required atomicity vs best-effort; enforce same-volume temporary placement, safe cleanup, fsync/rename choices, concurrency/race limits and no false crash-durability claims. Preserve clone/replace capabilities and native platform constraints without inventing portable APIs for every POSIX attribute. Preference bytes use Foundation property-list compatibility, not Keychain secrecy or immediate durable flush; report non-guaranteed atomicity when applicable.

## Residual work
- Audit remaining individual file plans before closing them. Retain unknown/error/scope/SDK requirements until they have named owner and evidence.
- Test file operations under real sandbox/process restart where possible; missing runtime checks must be reported, not silently passed.
- Keep provider-aware file coordination, user-selected document access and other host/UI-dependent functionality explicitly unsupported until separately implemented and tested.
- Move repeated native attribute parsing, safe path walkers and validation mechanics into reusable helpers *only after* proving semantic equivalence. No giant generalized native metadata dispatcher.

## Tests / handoff
Run portable tests, locked checks and strict Clippy for iOS device/simulator `ios-files` and `ios-preferences`, feature isolation, representative link/import audits, docs-check and zero-swift-source. Regression cases: symlinks, traversal, Unicode/names, identity races, type mismatch, max lengths, partial writes, concurrent modification and crash/atomicity wording. Original slice details remain reachable at the historical commit. Report code diff, test evidence limits, SHA.
