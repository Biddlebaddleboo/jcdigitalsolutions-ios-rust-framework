# B231: Mount-Relative File Path No-Go

## Candidate

`ATTR_CMNEXT_RELPATH` is a public extended-common attribute queried through `fgetattrlist`. It returns a path string for an opened filesystem object and could be mistaken for an app-relative file path.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_CMNEXT_RELPATH` as `0x00000004` in `sys/attr.h`; locked `libc` 0.2.190 binds the attribute and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as an `attrreference` to a UTF-8 NUL-terminated mount-relative path. The target path may be up to `PATH_MAX` bytes, and XNU warns of inconsistent behavior for hard-linked items, especially on filesystems without native `ATTR_CMN_PARENTID` support.
- The attribute is an extended-common `forkattr` queried with `FSOPT_ATTR_CMN_EXTENDED`. The public `fgetattrlist` declaration is available from iOS 3.0; the SDK provides no separate availability annotation for this field.
- `AppPath` already names an item relative to a caller-selected semantic app directory. The filesystem mount path is a physical container implementation detail, not a portable `AppPath` value or a current `FileBackend` input.

## Decision

No API change for B231. Exposing the mount-relative path would give callers a container-specific physical path with no supported operation in this facade, while creating a risk that callers persist or treat it as a stable app path. The documented hard-link ambiguity further prevents it from serving as a reliable per-entry path identity. Do not reconstruct or expose an `AppPath` from this value.

## Closure criteria

Revisit only if a concrete app-data operation requires this exact mount-relative path and Apple documents its sandbox-safe mapping, lifetime, and hard-link behavior for that use. A future host using `fgetattrlist` must also declare an applicable approved File Timestamp required-reason API entry in `PrivacyInfo.xcprivacy`.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.
