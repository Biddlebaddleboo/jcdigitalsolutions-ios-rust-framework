# B181: No-Go for Volume Hard-Link Support Snapshot

## Disposition

Do not add a public `volume_supports_hard_links` snapshot. Foundation exposes an honest volume
capability Boolean, but the portable `FileBackend` and `ios-files` have no operation that creates
a hard link or selects a hard-link strategy. A volume-level report would not help a supported
facade operation

## Candidate and evidence

- The candidate is Foundation's `NSURLVolumeSupportsHardLinksKey`, queried for one of the retained
  semantic app-directory roots. It reports whether the volume format supports hard links, not
  whether a particular file has multiple names or whether a future hard-link call will succeed.
- Apple's [`NSURLVolumeSupportsHardLinksKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportshardlinkskey?language=objc)
  defines a read-only Boolean `NSNumber` for whether a volume supports hard links.
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares the key available from iOS 4.0.
  Locked `objc2-foundation` 0.3.2 exposes the key constant; `ios-files` already has the `NSURL`
  resource-value API and `NSNumber` support. There is no binding or API-floor blocker.
- The portable `FileBackend` provides read, write, directory create/list, file/directory removal,
  and exists operations; it has no hard-link creation operation. `platform/ios/ios-files/src/lib.rs`
  has no `link`/`linkat` call. Its B105 `regular_file_hard_link_count` reports the actual
  `st_nlink` count for one regular file, which is more direct when a caller needs to know whether
  that entry is linked at observation time.
- A support Boolean would therefore expose a volume property without changing or guiding an
  operation in this facade. It would not prove that a given inode has aliases, that a link would
  remain within the sandbox, or that a native link attempt would pass permissions and policy.

## Closure criteria

Revisit only if a scoped safe hard-link operation or another concrete caller operation is added.
That design must state same-volume behavior, path containment, aliasing, and races before a
volume-support snapshot can select or inform it

## Validation

- Source review checked Apple's current `NSURLVolumeSupportsHardLinksKey` reference, the installed
  iPhoneOS 26.5 SDK header, the locked `objc2-foundation` 0.3.2 binding, the portable `FileBackend`
  method set, and the current `ios-files` source.
- Passed `git diff --check` and `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`.
- No tests, builds, runtime queries, probes, or live filesystem operations were run.
