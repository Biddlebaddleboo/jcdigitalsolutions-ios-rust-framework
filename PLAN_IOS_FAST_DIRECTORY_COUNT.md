# B140: Fast Directory-Entry Count No-Go

## Disposition

Do not add a second fast-count API for `NSURLDirectoryEntryCountKey` or `ATTR_DIR_ENTRYCOUNT` at this time. B128 already counts direct entries with the adapter's validated, no-follow directory descriptor; B131 stops at the first observed child for an empty/nonempty answer. The new Foundation/POSIX route has conditional availability and cost semantics that do not establish a reliable faster result on every supported volume

## Candidate and evidence

- The installed iPhoneOS 26.5 SDK declares `NSURLDirectoryEntryCountKey` from iOS 17.0. Its header comment says it counts stored filesystem objects, omits synthetic `.` and `..`, and returns `nil` if the URL is not a directory or the filesystem cannot cheaply compute the value.
- Apple's [directory entry count documentation](https://developer.apple.com/documentation/foundation/urlresourcevalues/directoryentrycount) repeats the optional result when the filesystem cannot cheaply compute the count.
- The installed `sys/attr.h` and locked `libc` 0.2.189 expose `ATTR_DIR_ENTRYCOUNT`, an unsigned 32-bit directory attribute, plus `fgetattrlist`.
- Apple's iOS [`getattrlist(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html) defines `ATTR_DIR_ENTRYCOUNT` as the number of filesystem objects in the directory, excluding synthetic items. It warns that this attribute is usually expensive on non-HFS Plus volumes and that not all volume formats support every attribute.
- `fgetattrlist` can query an already-open directory descriptor and avoid path re-resolution, but the documented cost warning still applies. The returned count is only `u_int32_t`; the docs do not define an overflow policy for directories above that range.
- Foundation's URL resource lookup could return the optional fast count, but `ios-files` path operations use validated descriptor-relative no-follow traversal. Re-resolving an `AppPath` to an `NSURL` could inspect a changed or symlink-replaced path instead of the directory inode already opened by the backend.
- Apple lists `fgetattrlist` and `getattrlist` in the required-reason Disk Space API category. This route therefore does not avoid B137's host privacy-manifest integration requirement.

## Why no implementation

A distinct utility would require a documented promise that the result is cheap or precomputed on the supported app-sandbox volume, plus exact overflow and unsupported-volume behavior. The current evidence does not provide that promise for `fgetattrlist`; Foundation's optional value also cannot be safely applied to an `AppPath` without a descriptor-bound URL route. B128 and B131 already provide honest direct count and early-stop semantics without a second API that may scan internally, return only a narrower count, or query a re-resolved path

## Closure criteria

Revisit only if Apple documents a descriptor-bound directory-count API with a reliable non-scanning or cheap-result guarantee on supported iOS app volumes, and defines the behavior for counts above `u32::MAX`. The app integration must also satisfy the required Disk Space privacy-manifest reason for any such API

## Validation

- No source, dependency, package, portable contract, guide API claim, or shared aggregate file changed.
- No tests, builds, probes, consumers, or live filesystem calls were run for this no-go.
- Apple docs, iOS 26.5 SDK headers, `objc2-foundation` 0.3.2 generated bindings, and `libc` 0.2.189 bindings were inspected.
