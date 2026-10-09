# B220: Recursive Directory Generation Count No-Go

## Candidate

`ATTR_CMNEXT_RECURSIVE_GENCOUNT` is a public `sys/attr.h` extended-common attribute queried through `fgetattrlist`. A directory's recursive count could look useful as a change snapshot for descendants, distinct from B128/B131 directory entry totals.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_CMNEXT_RECURSIVE_GENCOUNT` as `0x00000400` in the `forkattr` group. `fgetattrlist` is declared from iOS 3.0; the SDK has no separate availability annotation for this attribute.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines a `u_int64_t` recursive generation count for a directory marked `maintain-dir-stats` on APFS. The counter updates when a child changes; an unmarked directory returns zero.
- The public [XNU `sys/attr.h`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/attr.h) defines `ATTR_CMNEXT_SETMASK` as zero. The public header offers no settable extended-common attribute with which an app could mark an app-container directory. The SDK declares generic `fsetattrlist`, but the attribute's public contract does not define a setter for this count or its directory marker.
- No public iOS app API to mark an app-sandbox directory `maintain-dir-stats` was found. A query alone therefore cannot make normal app-owned directories produce a useful nonzero generation value.

## Decision

No API or dependency change for B220. Keep the candidate out of `ios-files`: the observable value is zero unless a directory was marked by an operation not exposed to this app-data facade. The implementation must not infer that zero means “unchanged,” “empty,” or “unsupported,” and must not substitute a file ID or modification time as a recursive generation counter.

## Closure criteria

Revisit only if Apple documents a public, app-callable way to mark a sandbox directory for recursive stats and establishes behavior, API floor, and volume support for iOS. The path must preserve the existing descriptor-relative no-follow contract, with no private APFS ioctl, privileged filesystem tool, or URL-based path re-resolution.

## Validation

This report uses installed public SDK headers and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.
