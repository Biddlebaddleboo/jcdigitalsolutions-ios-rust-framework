#ifndef FRAMEWORK_CPP_FRAMEWORK_HPP
#define FRAMEWORK_CPP_FRAMEWORK_HPP

#include <framework.h>

namespace framework {

struct AbiVersion final {
  uint32_t major;
  uint32_t minor;

  static AbiVersion current() noexcept {
    const uint64_t packed = framework_abi_version();
    return {static_cast<uint32_t>(packed >> 32), static_cast<uint32_t>(packed)};
  }
};

/** Move-only guard for a live `FrameworkOwnedBuffer` from the framework.
 * Do not change its fields or address; it must outlive this guard; use one
 * guard per descriptor.
 */
class OwnedBuffer final {
public:
  explicit OwnedBuffer(FrameworkOwnedBuffer &descriptor) noexcept
      : descriptor_(&descriptor) {}
  ~OwnedBuffer() noexcept { reset(); }

  OwnedBuffer(const OwnedBuffer &) = delete;
  OwnedBuffer &operator=(const OwnedBuffer &) = delete;

  OwnedBuffer(OwnedBuffer &&other) noexcept : descriptor_(other.descriptor_) {
    other.descriptor_ = nullptr;
  }

  OwnedBuffer &operator=(OwnedBuffer &&other) noexcept {
    if (this != &other) {
      reset();
      descriptor_ = other.descriptor_;
      other.descriptor_ = nullptr;
    }
    return *this;
  }

  const FrameworkOwnedBuffer *descriptor() const noexcept {
    return descriptor_;
  }
  const uint8_t *data() const noexcept {
    return descriptor_ == nullptr ? nullptr : descriptor_->data;
  }
  uint64_t size() const noexcept {
    return descriptor_ == nullptr ? 0 : descriptor_->length;
  }

private:
  void reset() noexcept {
    if (descriptor_ != nullptr) {
      framework_owned_buffer_destroy(descriptor_);
      descriptor_ = nullptr;
    }
  }

  FrameworkOwnedBuffer *descriptor_;
};

} // namespace framework

#endif
