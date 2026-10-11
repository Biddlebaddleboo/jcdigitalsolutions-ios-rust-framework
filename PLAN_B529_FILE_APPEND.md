# B529: Descriptor-relative file append

## Goal

Add a portable `framework-files` append operation and an iOS `ios-files` implementation for
binary logging and other append-only app data. The current facade only writes by creating or
replacing the full file, so it cannot preserve prior bytes without a read-and-rewrite cycle.

## Scope

- Add `FileBackend::append` and `Files::append` with no `std` dependency in `framework-files`.
- Implement iOS append with the existing descriptor-relative parent traversal, an existing-file
  `openat` using `O_APPEND | O_NOFOLLOW`, and exclusive `O_CREAT | O_EXCL` fallback to resolve
  simultaneous first appends without replacement.
- Reference `Files<IosFiles>::append` from the existing iOS app-data link probe so its code is
  retained in the existing device and simulator link checks.
- Add deterministic facade and native syscall regressions for binary contents, empty writes,
  create/existing-file mode behavior, final symlink and directory rejection, and concurrent
  append contiguity.
- Do not change Rideshare Bot, backup exclusion, validator profiles, capability status, or its
  migration map.

## Contract

- The path is relative to one existing application directory; missing parent directories are not
  created.
- An empty slice succeeds without opening or creating the final file.
- A missing final path creates a regular file with requested mode `0600`, subject to process umask.
  An existing regular file is opened without truncation or permission changes.
- The iOS backend rejects a final symlink via `O_NOFOLLOW` and rejects non-regular targets.
- Each successful call appends the complete borrowed slice contiguously at EOF. Concurrent
  successful calls do not overwrite one another; their order is unspecified.
- The iOS backend performs one `write` syscall (retrying only `EINTR`), so a short write returns an
  error rather than issuing another append syscall that could interleave with a concurrent call.
  An error can leave an unknown prefix or a newly created empty file.
- iOS creation retries a concurrent final-entry race up to eight open/create attempts, then
  returns the last native error; parent traversal is still descriptor-relative and no-follow.
- Appending is not a durability guarantee; no `fsync` is performed. If another actor removes or
  replaces the path after the descriptor opens, this call applies to the opened inode.

## Validation

- Run host tests for `framework-files` policy/facade semantics.
- Run host tests against the same descriptor-relative append helper used by `ios-files`.
- The existing spec has no standalone `ios-files` profile; it has an `ios-preferences` profile that
  runs the shared app-data link script. Root must register this package and regression file in an
  appropriate profile. Do not claim simulator or device runtime proof from compile/link checks.
