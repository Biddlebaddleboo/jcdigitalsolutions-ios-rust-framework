# B164: iOS App-Directory Filename-Component Limit Snapshot

## Disposition

Add a bounded query for the maximum filename-component length reported by the filesystem for direct children of one retained semantic app-directory root. This lets callers preflight names they plan to create directly under Documents, Caches, Temporary, or Application Support without guessing a cross-filesystem limit

## API

- `IosFiles::app_directory_name_max_bytes(&self, directory: AppDirectory) -> Result<Option<u64>, FileError>` calls `fpathconf(root_fd, _PC_NAME_MAX)` on the retained root descriptor.
- A nonnegative result is returned as `Some(value)` in bytes. POSIX `-1` with unchanged `errno` is returned as `None`, preserving the system's no-limit result. `-1` with nonzero `errno` is mapped through the existing POSIX error mapper; any other negative result maps to `InvalidInput`.
- The value applies to one filename component directly within the selected root. It is not a total path-length limit and does not establish the limit of a nested directory. The value is a snapshot, not a guarantee that a later create, rename, or other operation will succeed.
- `AppPath` still validates portable lexical rules only. This additive iOS-only query reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no portable `FileBackend` operation, framework, dependency, permission, or usage-description key.

## Platform evidence

- Apple's iOS [`fpathconf(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fpathconf.2.html) documents `fpathconf(int, int)`, defines `_PC_NAME_MAX` as the maximum number of bytes in a filename, and specifies the `-1`/`errno` distinction for an indeterminate or failed query.
- The installed iPhoneOS 26.5 SDK `unistd.h` declares `long fpathconf(int, int)` and `sys/unistd.h` defines `_PC_NAME_MAX` as `4`, without an explicit iOS availability annotation. This adds no explicit deployment-floor requirement above `ios-files`' iOS 10.0 baseline.
- Locked `libc` 0.2.189 exposes `fpathconf` as `c_long` and Apple `_PC_NAME_MAX` as `c_int` through its Unix/Apple bindings.
- No required-reason API, framework, entitlement, or privacy-manifest entry is introduced by `fpathconf` in this slice.

## Validation

- Do not add or run tests, execute consumers or probes, or perform live filesystem queries.
- Run `cargo fmt --package ios-files -- --check`.
- Run locked offline `cargo +1.94.1 check` and strict Clippy for `ios-files` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run iOS-target rustdoc, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.
