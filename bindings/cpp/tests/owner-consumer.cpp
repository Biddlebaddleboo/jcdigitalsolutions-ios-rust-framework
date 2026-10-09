#include <framework.hpp>

void release_on_scope_exit(FrameworkOwnedBuffer &descriptor) {
  framework::OwnedBuffer owner(descriptor);
  (void)owner;
}

int main() { return 0; }
