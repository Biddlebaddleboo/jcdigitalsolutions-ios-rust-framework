# B247: File Fork Count No-Go

## Audit result

Do not add an `ios-files` API for `ATTR_FILE_FORKCOUNT` in this slice. The public attribute returns
only a `u_int32_t` count of forks. It does not identify names or give each fork's size or contents.
The app-data adapter has no fork enumeration, resource-fork open/read, or fork-specific mutation
contract, so no supported operation can use the count to make a reliable decision.

B242 already exposes XNU's logical all-fork byte total, while B244 and B248 expose data-fork and
resource-fork allocated-byte snapshots. A count-only API would add little actionable information
and could be confused with hard-link count or the number of portable files. The portable
`framework-files::FileBackend` remains unchanged.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_FORKCOUNT` as a `u_int32_t`
  number of file forks and states that built-in filesystems support only data and resource forks;
  it does not make the count an enumeration or access API:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_FORKCOUNT` as `0x00000080` in
  `sys/attr.h`; locked `libc` 0.2.190 binds the attribute. The SDK provides no separate
  availability annotation beyond `fgetattrlist`'s iOS 3.0 declaration. Support is volume-dependent.
- XNU's public `getattrlist(2)` manual says not all volumes support all attributes:
  <https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2>

## Closure criterion

Revisit only if `ios-files` adds a separately scoped, safe fork enumeration/access operation with a
defined consumer for this count. Do not infer resource-fork existence, fork identity, or app data
access from a count alone.
