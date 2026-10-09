# iOS Speech authorization-status C ABI

The opt-in `ios-speech-status` feature exports one synchronous status read:

```c
FrameworkStatus framework_ios_speech_status_authorization_status(
    FrameworkIosSpeechAuthorizationStatus *out_raw_status);
```

`FrameworkIosSpeechAuthorizationStatus` is `int64_t`. Apple defines the named raw values as
`NOT_DETERMINED` 0, `DENIED` 1, `RESTRICTED` 2, and `AUTHORIZED` 3. Any other signed native
value is written unchanged so a newer OS value remains visible to the caller. Read the output
only when the function returns `FRAMEWORK_STATUS_OK`.

The query calls only `+[SFSpeechRecognizer authorizationStatus]`. It does not request permission,
show a prompt, create a recognizer, accept audio, or start recognition. `AUTHORIZED` reports saved
authorization only; it does not establish speech-service availability or recognition success.
The API floor is iOS 10.0. Below that floor the function returns `FRAMEWORK_STATUS_UNAVAILABLE`
with zero output. It adds no main-thread rule and makes no broad thread-safety promise.

The output pointer is required, caller-owned, writable, and aligned for `int64_t` during the
synchronous call. The API checks only nullness, initializes a valid output to zero before platform
handling, and does not retain it. The caller must prevent unsynchronized concurrent access during
the call.
A null pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A valid non-iOS call
returns `FRAMEWORK_STATUS_UNSUPPORTED` with zero output. A caught Rust panic returns
`FRAMEWORK_STATUS_PANIC` with zero output. The host app remains responsible for privacy metadata
needed by any permission or recognition flow outside this status-only API.

The C/C++ gates build and inspect host, device, and Simulator consumers; they do not execute a
consumer, linked probe, permission prompt, or Speech API call. See
[`PLAN_BINDINGS_F28.md`](../../PLAN_BINDINGS_F28.md) for measured imports, target minima, and
validation limits.
