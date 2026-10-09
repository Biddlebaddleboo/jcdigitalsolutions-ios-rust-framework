# B190: No-Go for Volume Sparse-File Support Snapshot

## Disposition

Do not add a public `volume_supports_sparse_files` snapshot. Foundation exposes a clear volume
Boolean, but neither `framework-files::FileBackend` nor `ios-files` has an operation that creates,
queries, or manages sparse regions. The Boolean cannot guide a supported facade operation.

## Candidate and evidence

- The candidate is Foundation's `NSURLVolumeSupportsSparseFilesKey`. Apple defines it as a
  read-only Boolean `NSNumber` for whether the volume format supports sparse files
  ([Apple API reference](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportssparsefileskey)).
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares the key at line 323 with
  `API_AVAILABLE(... ios(4.0) ...)`. The header defines sparse files as files with unwritten holes
  that do not consume disk space. Locked `objc2-foundation` 0.3.2 exposes
  `NSURLVolumeSupportsSparseFilesKey` under its `NSString` feature. No API or binding blocker
  exists.
- `framework-files::FileBackend::write` accepts an explicit byte slice. It has no sparse-file,
  hole, extent, or allocation operation. `ios-files` writes the caller's bytes and has no
  `ftruncate`, `lseek(SEEK_HOLE)`, or hole-punch API.
- B109's `regular_file_allocated_blocks_512` reports `st_blocks` for one regular file. That value
  does not prove the file has sparse regions; compression and allocation granularity can also make
  allocated size differ from logical size. The volume Boolean reports format support, not whether a
  particular file has holes or whether a later write will create them.

## Closure criteria

Revisit only if a concrete sparse-file operation is added, such as a bounded create-at-length or
extent query with explicit zero-read, allocation, and failure semantics. A volume support value
must not imply that a specific file is sparse or that a sparse write will succeed.

## Validation

- Source review checked Apple's API reference, the installed iPhoneOS 26.5 SDK header, the locked
  `objc2-foundation` 0.3.2 generated declaration, `framework-files::FileBackend`, and
  `platform/ios/ios-files/src/lib.rs`.
- No code, tests, builds, runtime queries, probes, or live filesystem operations were used.
