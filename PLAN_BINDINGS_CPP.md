# PLAN_BINDINGS_CPP.md — Workstream F3: Header-Only C++ Convenience Layer

## Status

F3's focused host gate `sh bindings/cpp/check.sh` passed. It used
`cargo build --locked --release -p framework-c-api`, checked C++ formatting, compiled and linked
`consumer.cpp` and `owner-consumer.cpp` against the real C API archive, and ran both executables.
It also compiled and ran `ownership.cpp` with its local test stub; static traits and runtime checks
confirmed move-only state, a pointer to the original descriptor, no descriptor change before
release, and one destroy call per descriptor. The `nm -u` checks found only
`framework_abi_version` for the core consumer, only `framework_owned_buffer_destroy` for the RAII
consumer, and no unresolved symbol for the test stub. The real-archive owner consumer proves
symbol resolution and process exit only; F1 has no core buffer creator, so runtime exactly-once
evidence comes from the test stub. `OwnedBuffer` requires a live framework-created descriptor whose
fields stay unchanged until C destroy; the descriptor must outlive the guard, with no second guard.
F3 links use `-nostdlib++`; no C++ runtime symbol is required, and no final-binary import audit is
claimed

F1 host ABI checks also passed against that locked archive: C11 and C++17 headers compiled, the C layout output matched the manifest, archive exports matched `c_symbols`, and `examples/c-minimal/main.c` linked and ran with output `framework ABI 1.0`

## Objective

Add a small C++17 convenience header over the existing stable core C ABI without a second implementation, hidden allocation, or new runtime dependency.

## Dependencies

Requires `PLAN_BINDINGS_CORE.md` and the integrated core C ABI.

## Write scope

- `bindings/cpp/**`
- C++ consumer checks and binding docs
- `Cargo.toml` only to exclude this header-only, non-Cargo directory from the `bindings/*` member glob

Do not change C ABI symbols/layouts, Rust capability APIs, workspace dependency versions/features, or platform backends.

## Required surface

- Add a header-only `framework.hpp` that includes the stable C header.
- Provide a small `AbiVersion` view over `framework_abi_version()`.
- Provide a move-only RAII owner for `FrameworkOwnedBuffer` that stores a pointer to the original descriptor and calls `framework_owned_buffer_destroy` exactly once.
- Preserve the C ABI rule: never copy or mutate an owned-buffer descriptor; the original descriptor must outlive its RAII view.
- Accept only a live framework-created descriptor; keep its fields and address unchanged until the guard calls C destroy, and permit no second live guard
- Follow an opt-in API's ownership signal; destroy a present empty value once, do not guard a default-empty absence output, and release a record field before API reuse
- Add no heap allocation, exception translation runtime, static registry, or C++ implementation source.
- Keep capability-specific wrappers out of this core slice.
- Include only `framework.h`; declare no opt-in C ABI symbols and enable no Cargo feature. F23's `framework_ios_key_support_p256_ecdsa_sha256_message_supported` remains in `framework_ios_key_support.h` behind `ios-key-support`
- F24's `framework_ios_mps_status_preferred_device_available` remains in `framework_ios_mps_status.h` behind `ios-mps-status`; it is outside F3 and uses scalar `uint8_t` output, not `FrameworkOwnedBuffer`
- F25's `framework_ios_videotoolbox_hardware_decode_supported` remains in `framework_ios_videotoolbox.h` behind `ios-videotoolbox`; callers use that C export directly and need no RAII wrapper
- F26's `framework_ios_camera_device_status_has_default_video_capture_device` remains in `framework_ios_camera_device_status.h` behind `ios-camera-device-status`; C++ callers use the scalar C export directly, with no RAII guard. It reports default-device presence only, not authorization or capture readiness
- F27's `framework_ios_core_ml_status_has_available_compute_device` remains in `framework_ios_core_ml_status.h` behind `ios-core-ml-status`; callers use that scalar C export directly and need no RAII guard. It reports a nonempty Core ML device-list snapshot only, not model or inference support. The C++ link uses `-nostdlib++`

## Validation and handoff

- Compile a C++17 consumer against the new header.
- Check move-only traits and single-destroy behavior with a test stub.
- Confirm the consumer links only the existing core C ABI symbols.
- Run the existing C ABI checks plus the focused C++ check, formatting, and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions.
