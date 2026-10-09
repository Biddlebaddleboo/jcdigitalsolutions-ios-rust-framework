# B205: No-Go for a Backup-Exclusion Setter on `AppPath`

## Candidate

Foundation exposes `NSURLIsExcludedFromBackupKey` as a read-write Boolean. Apple's [resource-key
documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/isexcludedfrombackupkey)
recommends it for caches and application-support files that are not needed in backups, and warns
that common operations on user documents can reset it to `false`. The iPhoneOS 26.5 SDK declares
the key available from iOS 5.1. `NSURL.setResourceValue:forKey:error:` is a URL-based setter;
`NSURL.h` declares it available from iOS 4.0.

## Disposition

Do not add a backup-exclusion mutation to `IosFiles`. `FileBackend` operations use validated
`AppPath` values and descriptor-relative no-follow traversal. Foundation's setter accepts a file
URL, not the already-open parent or file descriptor. Building an `NSURL` from the app-root path and
relative string would resolve the path again; a final or intermediate component could be replaced
with a symlink between validation and the Foundation call. A preliminary `fstatat`/`openat` check
cannot bind the later URL mutation to that checked entry.

No public descriptor-bound API for this Foundation backup property was found in the installed
SDK. `fsetxattr` exists, but the SDK does not define a supported xattr name/value mapping for
`NSURLIsExcludedFromBackupKey`; using a presumed private `com.apple.MobileBackup` representation
would not be an evidence-backed substitute. The B196 clone operation may also copy native extended
attributes, but this does not establish or promise backup-exclusion propagation.

## Closure criteria

- Reconsider only if Apple documents a public descriptor-relative setter for the backup property,
  or the `ios-files` contract gains a separately reviewed URL/path race policy for metadata
  mutation.
- Any later implementation must retain Apple's guidance to apply the property to cache or
  application-support data, not user documents, and must not claim the flag survives every file
  operation.
- No API, dependency, import, permission, entitlement, or matrix change is proposed here.

No build, test, link probe, consumer execution, runtime query, or live filesystem operation is part
of this feasibility note.
