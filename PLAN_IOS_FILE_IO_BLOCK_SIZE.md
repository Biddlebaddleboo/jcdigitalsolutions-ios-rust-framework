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

## B256: `ATTR_FILE_IOBLOCKSIZE` no-go

Do not add a second per-file I/O-size hint through `ATTR_FILE_IOBLOCKSIZE`. Apple's archived iOS
`getattrlist(2)` reference defines it as the optimal block size when reading or writing that file's
data. This has the same missing consumer as `st_blksize`: `ios-files` has no streaming descriptor,
caller buffer, or I/O policy hook. It is not a mandatory alignment or transfer size, and the facade
does not promise that returning it would improve performance.

The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_IOBLOCKSIZE` as `0x00000008` in
`sys/attr.h`; locked `libc` 0.2.190 binds it. The SDK gives no attribute-specific availability
annotation beyond `fgetattrlist` iOS 3.0, and actual support is filesystem-dependent. Any host use
would also require an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy`.

B121/B256 remain no-go until `ios-files` gains a documented file-I/O operation or caller policy
that can use a hint without treating it as a requirement or performance result. No source or public
API changed for B256.

## B260: `ATTR_FILE_CLUMPSIZE` no-go

Do not add `ATTR_FILE_CLUMPSIZE` as a per-file allocation policy value. The installed public SDK
marks this attribute obsolete. Apple's archived reference describes it as an allocation-clump hint
for the data fork, not a required allocation unit or guarantee that the filesystem will allocate
that amount. `ios-files` exposes no allocation-clump setter, streaming writer, or write policy hook
that could consume the hint.

The installed iPhoneOS 26.5 SDK defines the obsolete public macro as `0x00000010`; locked `libc`
0.2.190 binds the value. The SDK gives no attribute-specific availability annotation beyond
`fgetattrlist` iOS 3.0, and filesystem support is not guaranteed. No source or public API changed for
B260.
