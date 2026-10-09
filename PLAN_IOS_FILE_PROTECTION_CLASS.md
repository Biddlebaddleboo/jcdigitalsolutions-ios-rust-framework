# B168: No-Go for a Descriptor-Bound File-Protection-Class Snapshot

## Disposition

Do not add a public `ios-files` file-protection-class query with the currently verified Rust/Foundation surfaces. iOS provides a descriptor-level `F_GETPROTECTIONCLASS` command, but the installed public header exposes only an integer result and no numeric mapping to Foundation's documented protection values. A raw integer would not be a useful or honest protection-class contract

## Candidate and evidence

- The installed iPhoneOS 26.5 SDK `sys/fcntl.h` defines `F_GETPROTECTIONCLASS` as `63` and comments that it gets the file protection class from the extended attribute and returns `int`. The same header defines `F_SETPROTECTIONCLASS` as `64`; this audit considers no write operation.
- The installed SDK does not declare symbolic numeric values for protection classes. The inspected `libc` 0.2.189 Apple bindings expose generic `fcntl` but do not define `F_GETPROTECTIONCLASS` or a numeric class enum.
- Foundation publicly describes protection values as `NSString` values under `NSFileProtectionKey` or `NSURLFileProtectionKey`. `objc2-foundation` 0.3.2 generates these Foundation constants, but the documented Foundation attributes/resource-value APIs query a path/URL rather than the already-open descriptor used by `ios-files`.
- Re-resolving an `AppPath` through `NSFileManager` or `NSURL` would not preserve `ios-files`' retained-descriptor/no-follow path contract. Mapping the raw `F_GETPROTECTIONCLASS` integer to Foundation's named values would require an undocumented numeric mapping; exposing that integer as if it identified a class would be misleading.
- Apple documents that protection classes affect when file contents can be accessed (for example, while the device is locked). A class snapshot would not prove current data availability, a successful future read/write, or that an app's other access requirements are met.

## Decision

Keep this API out of `ios-files` until a public descriptor-bound API documents the mapping from `F_GETPROTECTIONCLASS`'s result to stable protection values. Do not hard-code private numeric values or re-resolve paths through Foundation. A future implementation needs a verified public mapping and a descriptor-preserving safe query path

## Platform evidence

- Apple's [Foundation `NSFileProtectionKey` documentation](https://developer.apple.com/documentation/foundation/fileattributekey/protectionkey?language=objc) says its corresponding value is an `NSString`; Apple's [`NSFileProtectionType` documentation](https://developer.apple.com/documentation/foundation/fileprotectiontype?language=objc) describes the named protection levels.
- Apple's [`URLResourceValues.fileProtection` documentation](https://developer.apple.com/documentation/foundation/urlresourcevalues/fileprotection) exposes the URL resource's protection level. The inspected API is URL-based, not descriptor-based.
- The installed iPhoneOS 26.5 SDK `sys/fcntl.h` contains the `F_GETPROTECTIONCLASS` macro. This confirms an SDK command definition, not a documented integer-to-protection-level mapping.
- This is a no-go report only. It adds no source/API, framework, dependency, permission, deployment-floor claim, or capability-matrix status change.

## Validation

- This audit is source/header/documentation only. Do not add or run tests, execute consumers or probes, or query live file protection state.
- Run `git diff --check` and a trailing-whitespace scan for this plan.
