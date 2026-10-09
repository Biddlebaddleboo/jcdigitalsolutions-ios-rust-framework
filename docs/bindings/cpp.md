# C++17 binding

The header-only C++17 layer in [`framework.hpp`](../../bindings/cpp/include/framework.hpp) wraps only the core C ABI. It adds no C++ source file, allocation, exception runtime, registry, dependency, or capability-specific API.

`framework.hpp` includes only `framework.h`; it declares no opt-in C ABI exports and enables no Cargo feature. F23's `framework_ios_key_support_p256_ecdsa_sha256_message_supported` remains in [`framework_ios_key_support.h`](../../bindings/c/include/framework_ios_key_support.h) behind `ios-key-support`; it is not part of this C++ layer and writes a scalar `uint8_t`, not a `FrameworkOwnedBuffer`. See the [F23 key-support guide](ios-key-support.md)

F24's `framework_ios_mps_status_preferred_device_available` remains in [`framework_ios_mps_status.h`](../../bindings/c/include/framework_ios_mps_status.h) behind `ios-mps-status`; it is not part of this C++ layer and writes a scalar `uint8_t`, not a `FrameworkOwnedBuffer`. See the [F24 MPS status guide](ios-mps-status.md)

F25's `framework_ios_videotoolbox_hardware_decode_supported` is available from [`framework_ios_videotoolbox.h`](../../bindings/c/include/framework_ios_videotoolbox.h) when the `ios-videotoolbox` Cargo feature is enabled. Call this scalar C export directly from C++; it returns no owned buffer or resource, so no `framework::OwnedBuffer` guard is needed. See the [F25 VideoToolbox guide](ios-videotoolbox.md)

F26's `framework_ios_camera_device_status_has_default_video_capture_device` is available from [`framework_ios_camera_device_status.h`](../../bindings/c/include/framework_ios_camera_device_status.h) when the `ios-camera-device-status` Cargo feature is enabled. Call this scalar C export directly from C++; it returns no owned buffer or resource, so no `framework::OwnedBuffer` guard is needed. It reports default-device presence only, not camera authorization or capture readiness. See the [F26 camera-device status guide](ios-camera-device-status.md)

F27's `framework_ios_core_ml_status_has_available_compute_device` is available from [`framework_ios_core_ml_status.h`](../../bindings/c/include/framework_ios_core_ml_status.h) when the `ios-core-ml-status` Cargo feature is enabled. Call this scalar C export directly from C++; its result owns no resource, so no `framework::OwnedBuffer` guard is needed. A true result reports only a nonempty Core ML device-list snapshot; it does not establish model or inference support. The F27 C++ link uses `-nostdlib++`. See the [F27 Core ML status guide](ios-core-ml-status.md)

`framework::AbiVersion::current()` decodes the major and minor fields returned by `framework_abi_version()`. `framework::OwnedBuffer` is a move-only release guard. It stores a pointer to the caller's original `FrameworkOwnedBuffer`; it does not copy or directly mutate the descriptor. Its `descriptor()`, `data()`, and `size()` methods provide read-only access.

The descriptor must be a live value created by a framework API, must remain at the same address, and must outlive its `OwnedBuffer`. Use at most one live guard for a descriptor, do not call the C destroy function while the guard is live, and do not alter the descriptor through another alias. When the guard is destroyed or replaced by move assignment, it calls `framework_owned_buffer_destroy` once for that descriptor. That C function releases the allocation and resets the original descriptor to null/zero. A moved-from guard does not release it.

`OwnedBuffer` is a generic guard, not a feature switch. An enabled C API may return a `FrameworkOwnedBuffer` inside an output record. Keep that record live at the same address through guard destruction; keep the descriptor unchanged until `framework_owned_buffer_destroy` runs, then destroy the guard before the API reuses the record

Use `OwnedBuffer` only after the API contract transfers ownership; buffer length alone does not signal absence. After `FRAMEWORK_STATUS_OK`, `framework_ios_secure_storage_read` transfers `out_secret`, and `framework_ios_preferences_get` transfers `out_value`, when `out_found == 1`; `framework_ios_clipboard_read` transfers `out_text` when `out_has_value == 1`. Guard and destroy each transferred descriptor exactly once, even when its length is zero. A zero presence byte leaves the default empty descriptor and does not transfer an owned value. For `framework_ios_file_provider_operation_poll`, keep `native_error_domain` empty before each poll; after call status `FRAMEWORK_STATUS_OK` and `out_ready == 1`, destroy a non-empty domain buffer before record reuse. Keep that record at its original address through guard destruction

```cpp
#include <framework.hpp>

void consume(FrameworkOwnedBuffer& output) {
    // output is the original live descriptor returned by a framework C API.
    framework::OwnedBuffer bytes(output);
    const uint8_t* data = bytes.data();
    const uint64_t length = bytes.size();
    (void)data;
    (void)length;
}
```

Build with `-std=c++17`, include `bindings/c/include` and `bindings/cpp/include`, and link the existing `framework-c-api` library. Run `sh bindings/cpp/check.sh` for compile, link, ABI-symbol import, move-only, and test-stub destruction checks.
