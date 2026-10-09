# B202: No-Go for a Volume Access-Permission Support Snapshot

## Candidate

Foundation exposes `NSURLVolumeSupportsAccessPermissionsKey`, a read-only Boolean that reports
whether a volume supports setting POSIX access permissions with `NSURLFileSecurityKey`. The
installed iPhoneOS 26.5 `NSURL.h` declares the key at iOS 11.0. Apple's [Foundation resource-key
documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsaccesspermissionskey)
defines the same narrow meaning.

## Disposition

Do not add a Rust-facing snapshot. The current `framework-files` and `ios-files` APIs do not set
POSIX permissions or ACLs. `IosFiles::entry_posix_permission_bits` only reads raw mode bits, and
`IosFiles::entry_effective_access` only queries the current process user's effective access mask;
neither consumes this volume capability value. Reporting it would add a support predicate without
an operation it can guide, as with the earlier B181, B184, B187, and B190 volume-support no-goes.

The Foundation setter accepts a URL rather than the backend's retained directory descriptor. A
future permission-mutation API would need its own path-confinement and race contract before this
volume flag could add useful guidance; this audit does not introduce such an operation or URL path.

## Evidence and closure criteria

- SDK declaration: `NSURLVolumeSupportsAccessPermissionsKey` is `API_AVAILABLE(ios(11.0))` and
  describes support for setting POSIX access permissions with `NSURLFileSecurityKey`.
- Existing implementation: the backend exposes no permission or ACL setter. Its permission APIs
  are read-only snapshots and retain the no-follow descriptor-relative path policy.
- No API, dependency, import, permission, entitlement, or matrix change is proposed.
- Reconsider only with a separately scoped, descriptor-safe permission mutation API and a concrete
  operation whose behavior depends on volume permission support.

No build, test, link probe, consumer execution, runtime query, or live filesystem operation is part
of this feasibility note.
