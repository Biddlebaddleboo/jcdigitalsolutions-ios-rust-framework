#include <framework.hpp>

#include <type_traits>

static_assert(!std::is_copy_constructible_v<framework::OwnedBuffer>);
static_assert(!std::is_copy_assignable_v<framework::OwnedBuffer>);
static_assert(std::is_nothrow_move_constructible_v<framework::OwnedBuffer>);
static_assert(std::is_nothrow_move_assignable_v<framework::OwnedBuffer>);

namespace {
uint32_t destroy_calls = 0;
FrameworkOwnedBuffer *last_destroyed = nullptr;
} // namespace

extern "C" uint64_t framework_abi_version(void) {
  return (uint64_t{1} << 32) | uint64_t{1};
}

extern "C" void framework_owned_buffer_destroy(FrameworkOwnedBuffer *buffer) {
  ++destroy_calls;
  last_destroyed = buffer;
  if (buffer != nullptr) {
    buffer->data = nullptr;
    buffer->length = 0;
    buffer->capacity = 0;
  }
}

int main() {
  const framework::AbiVersion version = framework::AbiVersion::current();
  if (version.major != 1 || version.minor != 1)
    return 1;

  uint8_t bytes[] = {4, 5, 6};
  FrameworkOwnedBuffer original{bytes, 3, 3};
  {
    framework::OwnedBuffer owner(original);
    if (owner.descriptor() != &original || owner.data() != bytes ||
        owner.size() != 3)
      return 2;
    if (original.data != bytes || original.length != 3 ||
        original.capacity != 3)
      return 3;
    framework::OwnedBuffer moved(static_cast<framework::OwnedBuffer &&>(owner));
    if (owner.descriptor() != nullptr || moved.descriptor() != &original ||
        destroy_calls != 0)
      return 4;
  }
  if (destroy_calls != 1 || last_destroyed != &original)
    return 5;
  if (original.data != nullptr || original.length != 0 ||
      original.capacity != 0)
    return 6;

  uint8_t left_bytes[] = {7};
  uint8_t right_bytes[] = {8, 9};
  FrameworkOwnedBuffer left{left_bytes, 1, 1};
  FrameworkOwnedBuffer right{right_bytes, 2, 2};
  {
    framework::OwnedBuffer left_owner(left);
    framework::OwnedBuffer right_owner(right);
    left_owner = static_cast<framework::OwnedBuffer &&>(right_owner);
    if (destroy_calls != 2 || left.data != nullptr || left.length != 0 ||
        left.capacity != 0)
      return 7;
    if (left_owner.descriptor() != &right ||
        right_owner.descriptor() != nullptr)
      return 8;
  }
  if (destroy_calls != 3 || last_destroyed != &right)
    return 9;
  if (right.data != nullptr || right.length != 0 || right.capacity != 0)
    return 10;
  return 0;
}
