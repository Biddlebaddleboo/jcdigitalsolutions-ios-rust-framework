# B425: iOS Volume Signature No-Go

## Candidate

`ATTR_VOL_SIGNATURE` returns a 32-bit volume signature. It may look like a compact volume or filesystem-format identifier.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_SIGNATURE` as `0x00000002` in `sys/attr.h`; locked `libc` 0.2.190 binds the same constant.
- Apple's XNU [`getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as `u_int32_t`, unique within a given filesystem type, and says it distinguishes volume formats handled by that filesystem. It does not define a stable volume identity, cross-mount lifetime, or an iOS format-to-value mapping.
- The query must include `ATTR_VOL_INFO`; public `fgetattrlist` is available from iOS 3.0, below `ios-files`' iOS 10.0 deployment floor. Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype) applies to `fgetattrlist` use.
- B158 already declines `f_type`, `f_fssubtype`, `f_fstypename`, and related `statfs` values as app-facing volume identity. The signature is a distinct raw value, but no current `ios-files` operation consumes a volume-format discriminator and the SDK/manual provide no iOS-specific mapping that would support a semantic enum.

## Decision

Do not add a raw volume-signature getter. Exposing an opaque `u32` would provide no supported `ios-files` operation, container identity, stable cross-mount key, or documented iOS format decision. Revisit only if a concrete caller operation needs to distinguish supported volume formats and Apple documents a usable mapping.

This audit adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement.

## Validation

Checked the installed iPhoneOS 26.5 SDK declarations, locked `libc` 0.2.190 bindings, and Apple's XNU manual. No build, test, runtime query, consumer, live filesystem call, or probe was run.
