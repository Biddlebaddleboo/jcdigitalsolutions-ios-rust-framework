# B168: No-Go for a Named File-Protection-Class Snapshot

## Disposition

This B168 no-go covers a named protection-level API based on `F_GETPROTECTIONCLASS` or Foundation's URL/path values. It does not cover B293's separate opaque `u32` from `ATTR_CMN_DATA_PROTECT_FLAGS`. The installed public header exposes only an integer result for `F_GETPROTECTIONCLASS` and no numeric mapping to Foundation's documented protection values; B168 therefore does not translate that result into a named class

## Candidate and evidence

- The installed iPhoneOS 26.5 SDK `sys/fcntl.h` defines `F_GETPROTECTIONCLASS` as `63` and comments that it gets the file protection class from the extended attribute and returns `int`. The same header defines `F_SETPROTECTIONCLASS` as `64`; this audit considers no write operation.
- The installed SDK does not declare symbolic numeric values for protection classes. The inspected `libc` 0.2.189 Apple bindings expose generic `fcntl` but do not define `F_GETPROTECTIONCLASS` or a numeric class enum.
- Foundation publicly describes protection values as `NSString` values under `NSFileProtectionKey` or `NSURLFileProtectionKey`. `objc2-foundation` 0.3.2 generates these Foundation constants, but the documented Foundation attributes/resource-value APIs query a path/URL rather than the already-open descriptor used by `ios-files`.
- Re-resolving an `AppPath` through `NSFileManager` or `NSURL` would not preserve `ios-files`' retained-descriptor/no-follow path contract. Mapping the raw `F_GETPROTECTIONCLASS` integer to Foundation's named values would require an undocumented numeric mapping; exposing that integer as if it identified a class would be misleading.
- Apple documents that protection classes affect when file contents can be accessed (for example, while the device is locked). A class snapshot would not prove current data availability, a successful future read/write, or that an app's other access requirements are met.

## Decision

Keep named protection-level mapping out of `ios-files` until a public descriptor-bound API documents the mapping from `F_GETPROTECTIONCLASS`'s result to stable protection values. B293 separately exposes the `ATTR_CMN_DATA_PROTECT_FLAGS` value as an opaque code only, with no level names or access/security inference. Do not hard-code private numeric mappings or re-resolve paths through Foundation

## Platform evidence

- Apple's [Foundation `NSFileProtectionKey` documentation](https://developer.apple.com/documentation/foundation/fileattributekey/protectionkey?language=objc) says its corresponding value is an `NSString`; Apple's [`NSFileProtectionType` documentation](https://developer.apple.com/documentation/foundation/fileprotectiontype?language=objc) describes the named protection levels.
- Apple's [`URLResourceValues.fileProtection` documentation](https://developer.apple.com/documentation/foundation/urlresourcevalues/fileprotection) exposes the URL resource's protection level. The inspected API is URL-based, not descriptor-based.
- The installed iPhoneOS 26.5 SDK `sys/fcntl.h` contains the `F_GETPROTECTIONCLASS` macro. This confirms an SDK command definition, not a documented integer-to-protection-level mapping.
- B168 itself added no source/API, framework, dependency, permission, deployment-floor claim, or capability-matrix status change. B293 is documented separately in [`PLAN_IOS_FILE_DATA_PROTECTION_CLASS_CODE.md`](PLAN_IOS_FILE_DATA_PROTECTION_CLASS_CODE.md).

## Validation

- This audit is source/header/documentation only. Do not add or run tests, execute consumers or probes, or query live file protection state.
- Run `git diff --check` and a trailing-whitespace scan for this plan.
