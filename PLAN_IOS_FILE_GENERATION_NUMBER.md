# B125: iOS `st_gen` File-Generation Number No-Go

## Disposition

Do not expose `st_gen` from `ios-files`. Apple's iOS `stat(2)` documentation limits the generation number to the superuser, so a normal sandbox app cannot promise a meaningful value

## Candidate

`st_gen` is a 32-bit generation field in the installed iOS `struct stat`. A generation value could appear to complement the B99 `(st_dev, st_ino)` snapshot by helping distinguish inode reuse, but the app-facing privilege boundary blocks an honest contract.

## Evidence and reason

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  states that `st_gen` is only available to the superuser.
- The installed iPhoneOS 26.5 SDK declares `st_gen` as `__uint32_t`; the locked `libc` Apple
  binding exposes `stat.st_gen: u32`. A binding alone does not establish app availability.
- The existing app-sandbox API has no privileged helper or entitlement that would grant
  superuser file metadata. Returning a possibly unavailable or meaningless number would invite a
  false persistent-identity claim and duplicate no supported user-space guarantee.
- B99 remains the honest `(st_dev, st_ino)` snapshot, with its inode-reuse and non-persistence
  caveats. B125 adds no source API, guide claim, dependency, or native operation.

## Closure criteria

Revisit only if Apple publishes a supported non-superuser API that guarantees this generation
field for app-sandbox files and defines its lifetime semantics
