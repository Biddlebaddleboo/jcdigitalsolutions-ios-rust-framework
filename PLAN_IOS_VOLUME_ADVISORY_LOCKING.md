# B187: No-Go for Volume Advisory-Locking Support Snapshot

## Disposition

Do not add a public volume advisory-locking support snapshot. Foundation exposes a clear Boolean,
but the portable `FileBackend` and `ios-files` have no lock operation or caller-owned file
descriptor API. The value cannot select or qualify a supported operation.

## Candidate and evidence

- The candidate is Foundation's `NSURLVolumeSupportsAdvisoryFileLockingKey`, queried for one
  retained semantic app-directory URL. Apple defines it as a read-only Boolean `NSNumber` for
  whole-file `flock`-style locks and the `O_EXLOCK` and `O_SHLOCK` flags of `open`
  ([Apple API reference](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsadvisoryfilelockingkey)).
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares the key at line 330 with
  `API_AVAILABLE(... ios(5.0) ...)`. Locked `objc2-foundation` 0.3.2 exposes
  `NSURLVolumeSupportsAdvisoryFileLockingKey` under its `NSString` feature. No API or binding
  blocker exists.
- `framework-files::FileBackend` has no lock, unlock, or open-file-handle method. The iOS adapter
  has no `flock`, `fcntl` lock command, `O_EXLOCK`, `O_SHLOCK`, or public descriptor escape. It
  therefore cannot apply the advertised lock style to a file in the sandbox facade.
- A volume-level `true` value would not acquire a lock, and would not establish that a particular
  file can be opened, that another process honors an advisory lock, or that a later operation
  succeeds. A `false` value has no effect on current facade calls.

## Closure criteria

Revisit only if a scoped file-lock operation is added. Its contract must define lock lifetime,
blocking behavior, cancellation, descriptor ownership, contention/error mapping, and whether the
lock coordinates with other app processes. Only then assess whether this volume Boolean helps a
caller choose that operation.

## Validation

- Source review checked Apple's API reference, the installed iPhoneOS 26.5 SDK header, the locked
  `objc2-foundation` 0.3.2 generated declaration, `framework-files::FileBackend`, and
  `platform/ios/ios-files/src/lib.rs`.
- No code, tests, builds, runtime queries, probes, or live filesystem operations were used.
