# B184: No-Go for Volume Symbolic-Link Support Snapshot

## Disposition

Do not add a public volume symbolic-link support snapshot. The Foundation key has a clear Boolean
meaning and an iOS 4.0 API floor, but neither `framework-files::FileBackend` nor `ios-files` creates
or follows symbolic links. The volume value cannot inform a supported operation.

## Candidate and evidence

- The candidate is Foundation's `NSURLVolumeSupportsSymbolicLinksKey`, queried for one retained
  semantic app-directory URL. Apple defines it as a read-only Boolean `NSNumber` that says whether
  the volume supports symbolic links
  ([Apple API reference](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportssymboliclinkskey)).
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares the key at line 319 with
  `API_AVAILABLE(... ios(4.0) ...)`. Locked `objc2-foundation` 0.3.2 exposes
  `NSURLVolumeSupportsSymbolicLinksKey` under its `NSString` feature. No API or binding blocker
  exists.
- `framework-files::FileBackend` has no symbolic-link create, read, or follow operation. The iOS
  adapter uses no-follow path traversal; `entry_kind` classifies a final symbolic link as `Other`,
  and `remove_file` removes a final entry without following it. The adapter has no symlink create,
  read-link, or link operation.
- A `true` volume value would not relax the no-follow policy; `false` would not change any current
  facade result. The key does not report whether a specific path is a link, prove containment, or
  guarantee a later native link operation.

## Closure criteria

Revisit only if a concrete symbolic-link operation is added to the portable contract or to a
separate, scoped iOS API. That work must define target interpretation, sandbox containment,
no-follow behavior, and namespace-race limits before this volume property can inform it.

## Validation

- Source review checked Apple's API reference, the installed iPhoneOS 26.5 SDK header, the locked
  `objc2-foundation` 0.3.2 generated declaration, `framework-files::FileBackend`, and
  `platform/ios/ios-files/src/lib.rs`.
- No code, tests, builds, runtime queries, probes, or live filesystem operations were used.
