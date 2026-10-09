# iOS Natural Language contextual-model asset C ABI

The opt-in `ios-natural-language-status` feature exports one synchronous status query:

```c
FrameworkStatus framework_ios_natural_language_english_contextual_embedding_assets(
    FrameworkIosNaturalLanguageAssetStatus *out_status);
```

`FrameworkIosNaturalLanguageAssetStatus` is `uint32_t`, not an Apple or Rust enum layout. On
`FRAMEWORK_STATUS_OK`, its value is one of the five fixed F29 codes:

| Code | Meaning |
| --- | --- |
| `UNAVAILABLE` (0) | B59's iOS 17.0 API guard did not pass |
| `LANGUAGE_UNAVAILABLE` (1) | The generated `NLLanguageEnglish` constant has no value |
| `NO_MODEL` (2) | Apple returned no `NLContextualEmbedding` object |
| `ASSETS_NOT_AVAILABLE` (3) | B59 read `hasAvailableAssets == false` |
| `ASSETS_AVAILABLE` (4) | B59 read `hasAvailableAssets == true` |

These codes map B59's five Rust enum variants. Code 0 is a successful B59 result below iOS 17.0,
not `FRAMEWORK_STATUS_UNAVAILABLE`. A valid non-iOS call returns
`FRAMEWORK_STATUS_UNSUPPORTED` and output zero. A null output pointer returns
`FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A caught Rust panic returns
`FRAMEWORK_STATUS_PANIC` with output zero. Read the output only for `FRAMEWORK_STATUS_OK`.

B59 creates a temporary `NLContextualEmbedding`, checks only whether the English model assets are
on-device, then releases the object before return. It does not load or unload the model, accept
text, compute vectors, request or download assets, or guarantee a future load or vector result.
The API floor is iOS 17.0. Calls are synchronous on the caller's thread; no main-thread rule or
broader thread-safety promise is added.

The output is caller-owned, valid, properly aligned, writable `uint32_t` storage for the full
synchronous call. The caller must prevent unsynchronized concurrent access. The function checks
only nullness. The output is initialized to zero before platform handling, and the function does not
retain the output pointer.
The C/C++ gates build and inspect host/device/Simulator consumers, but do not execute a consumer,
probe, model query, asset request, or Natural Language operation. See
[`PLAN_BINDINGS_F29.md`](../../PLAN_BINDINGS_F29.md) for imports, deployment minima, and gate limits.
