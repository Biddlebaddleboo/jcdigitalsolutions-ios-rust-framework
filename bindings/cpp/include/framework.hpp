#ifndef FRAMEWORK_CPP_FRAMEWORK_HPP
#define FRAMEWORK_CPP_FRAMEWORK_HPP

#include <framework.h>
#include <string_view>

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

  /** Adopt only when a C API explicitly transferred ownership.
   * Use the API's presence/ownership output; buffer length is not a signal.
   */
  static OwnedBuffer from_transfer(FrameworkOwnedBuffer &descriptor,
                                   bool ownership_transferred) noexcept {
    return OwnedBuffer(ownership_transferred ? &descriptor : nullptr);
  }

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
  explicit OwnedBuffer(FrameworkOwnedBuffer *descriptor) noexcept
      : descriptor_(descriptor) {}

  void reset() noexcept {
    if (descriptor_ != nullptr) {
      framework_owned_buffer_destroy(descriptor_);
      descriptor_ = nullptr;
    }
  }

  FrameworkOwnedBuffer *descriptor_;
};

struct ErrorDetailView final {
  FrameworkStatus status;
  std::string_view message;
};

/** Move-only owner for one live `FrameworkErrorDetailHandle`.
 * The message returned by `view()` is borrowed and remains valid only while
 * this owner remains alive.
 */
class ErrorDetail final {
public:
  explicit ErrorDetail(FrameworkErrorDetailHandle detail) noexcept
      : detail_(detail) {}

  ~ErrorDetail() noexcept { reset(); }

  ErrorDetail(const ErrorDetail &) = delete;
  ErrorDetail &operator=(const ErrorDetail &) = delete;

  ErrorDetail(ErrorDetail &&other) noexcept : detail_(other.detail_) {
    other.detail_ = nullptr;
  }

  ErrorDetail &operator=(ErrorDetail &&other) noexcept {
    if (this != &other) {
      reset();
      detail_ = other.detail_;
      other.detail_ = nullptr;
    }
    return *this;
  }

  FrameworkStatus view(ErrorDetailView &out_view) const noexcept {
    out_view = {};
    if (detail_ == nullptr) {
      return FRAMEWORK_STATUS_INVALID_ARGUMENT;
    }
    FrameworkStatus status = FRAMEWORK_STATUS_OK;
    FrameworkStr message{};
    const FrameworkStatus result =
        framework_error_detail_view(detail_, &status, &message);
    if (result != FRAMEWORK_STATUS_OK) {
      return result;
    }
    if (message.length == 0) {
      out_view = {status, {}};
      return FRAMEWORK_STATUS_OK;
    }
    if (message.data == nullptr) {
      return FRAMEWORK_STATUS_INTERNAL_ERROR;
    }
    out_view = {status,
                std::string_view(reinterpret_cast<const char *>(message.data),
                                 static_cast<size_t>(message.length))};
    return FRAMEWORK_STATUS_OK;
  }

private:
  void reset() noexcept {
    if (detail_ != nullptr) {
      framework_error_detail_destroy(detail_);
      detail_ = nullptr;
    }
  }

  FrameworkErrorDetailHandle detail_;
};

} // namespace framework

#endif
