# iOS sandbox files

## Scope and setup

`ios-files` implements `framework_files::FileBackend` as a caller-owned `IosFiles` value. It has
no global registration, executor, callback, or permission prompt. Construct it with
`IosFiles::new()` and pass it to `framework_files::Files`.

The backend resolves Documents, Caches, and Application Support with Foundation's public
[`NSFileManager` directory URL API](https://developer.apple.com/documentation/foundation/nsfilemanager)
`URLForDirectory:inDomain:appropriateForURL:create:error:`. It resolves Temporary with
[`NSFileManager.temporaryDirectory`](https://developer.apple.com/documentation/foundation/filemanager/temporarydirectory).
Each URL is opened once with a directory/no-follow open, and the backend retains the descriptor
for its lifetime. These are app-sandbox locations; this backend does not
accept user-selected document-provider URLs, security-scoped bookmarks, iCloud container URLs, or
file-provider URLs. Provider-aware coordination is unsupported.

The backend's API floor is iOS 10.0. The Xcode 26.5 iOS SDK headers declare
`renameatx_np`, `NSFileManager.temporaryDirectory`, and both volume rename-support resource keys
available from iOS 10.0. The crate does not set a deployment target; a device app must select iOS
10.0 or later. The link probe's device executable recorded minos 10.0. The Rust arm64 simulator
target recorded minos 14.0 in `LC_BUILD_VERSION`, despite the same deployment-target environment
value. These are SDK/header and binary load-command records, not runtime tests.

No Info.plist usage-description key, permission, or entitlement is required for these ordinary
sandbox directories. The app sandbox and standard iOS file-protection policy still apply.

## Path and symlink policy

Every call rechecks the portable slash-separated relative-path rules at the native boundary: no
empty path, leading/trailing slash, empty segment, `.` or `..` segment, backslash, or NUL. Each
component is passed separately to descriptor-relative POSIX calls. Parent traversal uses
`openat` with `O_DIRECTORY | O_NOFOLLOW`; final file opens also use `O_NOFOLLOW`. This avoids
string-prefix checks and prevents intermediate or final symlinks from redirecting reads, writes, or
directory traversal outside the selected root. The `libc` dependency is private to this crate and
provides the POSIX declarations/constants needed for this boundary; Foundation remains the narrow
public API for sandbox URL resolution.

`read` rejects a final symlink, `read_directory` lists a symlink as `FileKind::Other`, and
`exists` reports a final symlink as present without following it. `remove_file` unlinks a final
symlink itself; `remove_directory` does not follow one. Create-or-replace atomically replaces a
final symlink rather than following it. Filesystem names that are not valid UTF-8 cannot be
represented by the portable `DirectoryEntry` contract and cause `InvalidInput`.

## Writes and errors

Create-or-replace writes to a same-directory temporary file and commits with POSIX `renameat`.
Create-new uses Darwin `renameatx_np(RENAME_EXCL)`. Replace-existing uses
`renameatx_np(RENAME_SWAP)` so a missing target is not created and the old file is replaced at one
atomic visibility point. The backend queries `NSURLVolumeSupportsExclusiveRenamingKey` and
`NSURLVolumeSupportsSwapRenamingKey` for each root before it accepts these extended rename modes.
If a volume reports no support, `RequireAtomic` returns `Unsupported` before target mutation;
`AllowNonAtomic` uses exclusive direct creation or open/truncate/write and reports
`NotGuaranteed`. Such a failed fallback write may leave a new partial file or partially changed
bytes, as the portable contract allows for non-atomic writes.

Successful staged writes report `Atomic`; this means readers see the whole old file or whole new
file, not that bytes or directory metadata are durable after a crash. The backend does not call
`fsync`. After a successful `RENAME_SWAP`, it unlinks the old version at the staging name on a
best-effort basis; a cleanup failure does not undo or turn the committed write into an error and
can leave a hidden `.ios-files-*` entry. New files use mode `0600`; new directories use mode
`0700`. Parent directories are not created implicitly.

POSIX `errno` is preserved as `PlatformErrorCode`; common values map to the portable
`NotFound`, `AlreadyExists`, `PermissionDenied`, `InvalidInput`, `Unsupported`, or
`ResourceExhausted` category. Foundation directory lookup errors map to `Platform` and preserve
`NSError.code`; its error domain is not represented by the current portable error type.

## Cost and execution behavior

Methods are synchronous and can block on filesystem I/O. They start no callback, worker thread, or
async task; dropping `IosFiles` does not cancel a call already on the stack. `read` copies file
bytes into its returned `Vec`; `write` passes the caller's slice to POSIX writes without an
intermediate Rust byte buffer. Path components are copied into temporary C strings. Directory
entry names are copied from `readdir` into owned Rust `String` values. File operations expose no
native root path or descriptor escape handle.

The iOS dependency surface is `objc2` 0.6.5 and `objc2-foundation` 0.3.2 with only the Foundation
features used for directory lookup and volume metadata, plus `libc` for POSIX calls. A minimal
consumer of both app-data crates was built for device and simulator with
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --release --target aarch64-apple-ios` and
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --release --target aarch64-apple-ios-sim`. `otool -L`
showed Foundation, CoreFoundation, `libobjc.A`, `libSystem.B`, and `libiconv.2`; neither binary
imports UIKit, Network, Swift, Python, or another capability framework. This link probe used Xcode
26.6 with the iOS 26.5 SDK, below the planned Xcode 27.x baseline. It proves link imports only; no
simulator launch or live sandbox operation was performed.
