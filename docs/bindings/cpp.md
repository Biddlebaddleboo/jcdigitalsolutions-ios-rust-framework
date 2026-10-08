# C++17 binding

The header-only C++17 layer in [`framework.hpp`](../../bindings/cpp/include/framework.hpp) wraps only the core C ABI. It adds no C++ source file, allocation, exception runtime, registry, dependency, or capability-specific API.

`framework::AbiVersion::current()` decodes the major and minor fields returned by `framework_abi_version()`. `framework::OwnedBuffer` is a move-only release guard. It stores a pointer to the caller's original `FrameworkOwnedBuffer`; it does not copy or directly mutate the descriptor. Its `descriptor()`, `data()`, and `size()` methods provide read-only access.

The descriptor must be a live value created by a framework API, must remain at the same address, and must outlive its `OwnedBuffer`. Use at most one live guard for a descriptor, do not call the C destroy function while the guard is live, and do not alter the descriptor through another alias. When the guard is destroyed or replaced by move assignment, it calls `framework_owned_buffer_destroy` once for that descriptor. That C function releases the allocation and resets the original descriptor to null/zero. A moved-from guard does not release it.

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
