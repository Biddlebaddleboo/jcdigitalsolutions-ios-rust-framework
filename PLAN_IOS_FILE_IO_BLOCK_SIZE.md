# B121: iOS `st_blksize` I/O-Hint No-Go

## Disposition

Do not add a public `ios-files` API for `st_blksize` at this time. The field is an optimal-I/O-block-size hint, but `ios-files` exposes no streaming descriptor or caller buffer policy to which that hint can be applied

## Candidate

`st_blksize` is available in the same `stat` result as the other B-series metadata and could be
returned as a positive `i32` for one entry. It is distinct from the B109 allocated-block count and
logical file size, but its current utility is not sufficient for a public facade method.

## Evidence and reason

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_blksize` as the optimal I/O block size for a file. It does not promise that the value
  is a mandatory alignment, a transfer size, or a performance result.
- The installed iPhoneOS 26.5 SDK exposes `st_blksize` as `blksize_t`; the locked `libc` Apple
  binding exposes it as `i32`.
- Current `ios-files` reads return an owned `Vec<u8>` or write through its own synchronous backend.
  There is no public streaming file handle, caller-provided buffer, or I/O policy hook that could
  consume this hint. A standalone number would have little actionable utility and could be
  misread as a required block or buffer size.
- `st_gen` is not an alternative: Apple's same manual says the generation number is available only
  to the superuser. `st_rdev` is a device-type field for special files, which the app-data facade
  does not treat as regular user file content.

## Closure criteria

Revisit only if `ios-files` gains a streaming descriptor or a documented caller buffer policy that
can use this hint, and if the adapter can map the hint without claiming alignment or performance
guarantees

## Validation

- B121 changes no code, Cargo manifest, portable contract, or guide behavior.
- The SDK/header facts above were inspected; scoped `git diff --check` passed with the B118 work.
- No tests, builds, probes, consumers, or live filesystem calls were run specifically for B121.
