# B165: No-Go for an AppPath Length-Limit Query

## Disposition

Do not add an `ios-files` method that reports `_PC_PATH_MAX` for a semantic app-directory root. The value describes a pathname byte limit, but `ios-files` does not submit the full `AppPath` to one pathname-based system call. It walks validated components with descriptor-relative `openat`, so a whole-path maximum would not describe the adapter's operation and could mislead callers

## Candidate and evidence

- Apple's iOS [`fpathconf(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fpathconf.2.html) defines `_PC_PATH_MAX` as the maximum number of bytes in a pathname and documents `fpathconf` for an open descriptor.
- The installed iPhoneOS 26.5 SDK `sys/unistd.h` defines `_PC_PATH_MAX` as `5`; libc 0.2.189 exposes the corresponding Apple constant and `fpathconf` binding. The candidate is technically callable, but its reported value does not match this adapter's traversal semantics.
- `AppPath` is split into components. `open_directory` opens each parent component with a separate `openat` relative to the current directory descriptor; final-entry operations use the same parent-descriptor plus one-component pattern. The full relative path is never passed as one OS pathname. Per-component byte limits are addressed separately by B164's `_PC_NAME_MAX` query for direct children of a retained semantic root.
- This method does not make a claim about arbitrary Foundation URLs, paths outside the app sandbox, or nested-directory name limits.

## Decision

Keep `_PC_PATH_MAX` out of the public API. It would not let callers predict success of `AppPath` traversal. A proposed future path-length API would need a concrete syscall or platform contract that consumes the whole path and a useful caller decision that the existing component-wise operations cannot answer.

## Validation

- This is a no-go report only. Do not add or run tests, execute consumers or probes, or perform live filesystem calls.
- Run `git diff --check` and a trailing-whitespace scan for this plan.
