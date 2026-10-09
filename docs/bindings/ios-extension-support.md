# iOS extension metadata C ABI

The opt-in `ios-extension-support` feature exports one synchronous metadata read:

```c
FrameworkStatus framework_ios_extension_support_read_extension_point_identifier(
    FrameworkStr bundle_path,
    FrameworkIosExtensionMetadataError *out_error,
    FrameworkOwnedBuffer *out_identifier);
```

The input must be valid immutable UTF-8 for the call. On iOS, B77 accepts only an absolute path
whose last component has a non-empty `.appex` suffix. It reads only
`NSExtension.NSExtensionPointIdentifier`; it does not load extension code, search for a bundle,
or validate the schema of a particular extension point.

`FRAMEWORK_STATUS_OK` means the read completed. The error output maps B77's eight metadata outcomes:

| Code | Meaning |
| --- | --- |
| `NONE` (0) | The identifier buffer contains the copied UTF-8 value |
| `INVALID_BUNDLE_PATH` (1) | Path is relative, contains NUL, or does not end in a nonempty `.appex` name |
| `BUNDLE_UNAVAILABLE` (2) | Foundation could not create a bundle |
| `INFO_DICTIONARY_UNAVAILABLE` (3) | Foundation did not provide bundle metadata |
| `MISSING_EXTENSION_DICTIONARY` (4) | The information dictionary has no `NSExtension` value |
| `INVALID_EXTENSION_DICTIONARY_TYPE` (5) | The `NSExtension` value is not a dictionary |
| `MISSING_POINT_IDENTIFIER` (6) | The extension dictionary has no point identifier |
| `INVALID_POINT_IDENTIFIER_TYPE` (7) | The point identifier is not a string |
| `EMPTY_POINT_IDENTIFIER` (8) | The point identifier string is empty |

On a non-NONE error code, the output buffer remains empty. A malformed input span, invalid UTF-8,
null output, or overlapping input/output range returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`.
The API initializes valid, disjoint outputs before platform handling. A valid non-iOS call returns
`FRAMEWORK_STATUS_UNSUPPORTED` with zero outputs; a caught Rust panic returns
`FRAMEWORK_STATUS_PANIC`. `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED` means the result could not be
represented by the ABI buffer.

On success, `out_identifier` owns length-delimited UTF-8 bytes. The caller must pass an empty
descriptor on entry and destroy a returned descriptor exactly once, unchanged, with
`framework_owned_buffer_destroy`. Output storage must be writable, aligned, and disjoint from
each other and the input. The API retains no pointer or Foundation object.

The Foundation API floor is iOS 4.0. Release link probes use device minos 12.0 and Simulator minos
14.0. These newer probe settings do not change the API floor. Reading this key does not prove an
extension is installed, registered, signed, enabled, approved, entitled, launchable, or compatible
with a host. It adds no App Intents, .appext, build-host plist, or extension lifecycle support.
See [PLAN_BINDINGS_F30.md](../../PLAN_BINDINGS_F30.md) for imports, gates, and evidence limits.
