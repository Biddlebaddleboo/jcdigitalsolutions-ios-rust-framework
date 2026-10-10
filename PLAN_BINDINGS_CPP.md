# PLAN_BINDINGS_CPP.md — Workstream F3: Header-Only C++ Convenience Layer

## Status

The prior F3 host gate `sh bindings/cpp/check.sh` passed at ABI 1.0. It compiled and linked
`consumer.cpp` and `owner-consumer.cpp` against the C API archive, ran both executables, and compiled
and ran `ownership.cpp` with its local destroy stub. Static traits and runtime checks confirmed
move-only state, a pointer to the original descriptor, no descriptor change before release, and one
destroy call per descriptor. The `nm -u` checks found only `framework_abi_version` for the core
app, only `framework_owned_buffer_destroy` for the RAII app, and no unresolved symbol for the stub.
This is ABI 1.0 evidence only. The version fixtures accept major 1 with minor 1 or later, so
additive ABI 1.x releases do not fail an exact-minor check. The ownership stub reports ABI 1.1.
`OwnedBuffer` still requires a live framework-created descriptor whose fields stay unchanged until
C destroy; the descriptor must outlive the guard, with no second guard. The prior F3 gate used
`-nostdlib++`; no
C++ runtime symbol was required, and that gate made no final-binary import claim

F1's ABI 1.0 host app linked and ran with output `framework ABI 1.0`. ABI 1.1 adds
`framework_options_v1_validate`; the F1 plan records C11/C++17 header and manifest layout checks
plus ABI 1.1 C/C++ link-only checks; no ABI 1.1 app ran. This audit adds C++17 compile-only coverage for all five C++ fixture
sources. `bindings/c/tests/header_cpp.cpp` asserts the validator signature and the 16-byte,
4-aligned `FrameworkOptionsV1` layout with offsets 0, 4, 8, and 12; these values match
`bindings/c/abi-manifest.json`

`clang++ -std=c++17 -fno-exceptions -fno-rtti -Wall -Wextra -Werror -pedantic -I bindings/c/include -I bindings/cpp/include -fsyntax-only` passed for `bindings/cpp/tests/consumer.cpp`, `bindings/cpp/tests/owner-consumer.cpp`, `bindings/cpp/tests/ownership.cpp`, `bindings/c/tests/header_cpp.cpp`, and `bindings/c/tests/secure_storage_header_cpp.cpp`. `format_tool=$(xcrun -f clang-format); "$format_tool" --dry-run --Werror bindings/c/tests/header_cpp.cpp` and `git diff --check -- PLAN_BINDINGS_CPP.md bindings/c/tests/header_cpp.cpp` passed. After the fresh locked ABI 1.1 release build recorded in `PLAN_BINDINGS_CORE.md`, a temporary C++17 app that calls `framework_options_v1_validate` compiled and linked to `target/abi11-fresh-target/release/libframework_c_api.a` with `-nostdlib++`; its object had only `_framework_options_v1_validate` undefined, and the linked binary defines that symbol with no `nm -u` output. `otool -L` lists only `/usr/lib/libSystem.B.dylib`; `otool -Iv` lists no import records. The app source stayed in `target/abi11-link-only`; no checked-in link-only gate was added. No F3 implementation or contract gap was found. The post-ABI-1.1 `sh bindings/cpp/check.sh` gate remains unverified; it runs the ABI-version app, RAII app, and ownership stub. Xcode 27 workflow run `38040954950` passed the corrected C API gate at step 234, then failed at step 235 (`Header-only C++17 binding check`). The preceding C API log reports ABI 1.3, while both C++ version fixtures required exact minor 1; those guards now accept later minor versions within major 1. The hosted C++ gate must be rerun to confirm the linked consumers. No post-correction app execution or runtime proof is claimed

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
