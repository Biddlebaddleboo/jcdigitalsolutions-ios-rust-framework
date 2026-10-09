# B143: iOS Raw Volume-Free Blocks No-Go

## Disposition

Do not expose `statfs.f_bfree` as an app-data capacity API. B137 returns `f_bavail`, the free blocks Apple documents as available to non-superusers. `f_bfree` also counts free blocks that may not be available to an ordinary sandbox process, so it does not improve an app's storage display or write decision

## Candidate and evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_bfree` as free blocks in the filesystem and `f_bavail` as free blocks available to non-superusers.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` and locked `libc` 0.2.189 Apple binding expose both fields as `u64` in `statfs`.
- `ios-files` has no privileged helper or entitlement that lets an app consume blocks reserved from non-superusers. B137 already returns the field aligned with an ordinary app's documented privilege class.

## Why no implementation

Returning `f_bfree` beside B137 could invite callers to treat reserved or otherwise unavailable filesystem space as usable. It does not establish an app quota, reserve capacity, or predict whether a later write succeeds. The extra raw number has no separate supported app-data operation

## Closure criteria

Revisit only if Apple documents an app-visible use for the distinction between `f_bfree` and `f_bavail` that does not imply the app can consume reserved blocks

## Validation

- No source, dependency, package, portable contract, guide API claim, or shared aggregate file changed.
- Apple's iOS `statfs(2)` manual, iPhoneOS 26.5 `sys/mount.h`, and locked `libc` 0.2.189 bindings were inspected.
- No tests, builds, probes, consumers, or live filesystem calls were run for this no-go.
