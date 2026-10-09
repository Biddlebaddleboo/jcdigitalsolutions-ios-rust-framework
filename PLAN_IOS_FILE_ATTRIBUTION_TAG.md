# B232: iOS File Attribution-Tag No-Go

## Candidate

`ATTR_CMNEXT_ATTRIBUTION_TAG` is a public `sys/attr.h` extended-common attribute that appears to expose a numeric owner tag for a filesystem object. A descriptor-bound snapshot could be added without reading file contents.

## Evidence

- The installed iPhoneOS 26.5 SDK defines `ATTR_CMNEXT_ATTRIBUTION_TAG` as `0x00000800` in the public `sys/attr.h` `forkattr` group. Locked `libc` 0.2.190 does not bind this macro; the generic `fgetattrlist` API remains available.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the value as an optional `u_int64_t` ID representing the bundle ID owner associated with the file, with zero meaning the file is not attributed yet.
- The public [XNU `sys/attr.h`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/attr.h) defines `ATTR_CMNEXT_SETMASK` as zero. No public app API to set this file attribution tag was found.
- The inspected public contract does not provide a mapping from the returned numeric ID to a bundle identifier, explain whether app-container files receive tags, or define a consumer operation in the existing `ios-files` facade.

## Decision

No API or dependency change for B232. A raw number with no public bundle-ID mapping, no app-container behavior contract, and no supported caller action would imply provenance utility that the available evidence does not establish. Do not infer “owned by this app” from zero or any nonzero value.

## Closure criteria

Revisit only if Apple documents the tag's behavior for ordinary app-sandbox files and provides a public mapping or operation that gives callers a stable use for the numeric value. Any implementation must remain descriptor-bound, distinguish absent/zero/unsupported states, and include the host's required-reason `PrivacyInfo.xcprivacy` obligation for `fgetattrlist`.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.
