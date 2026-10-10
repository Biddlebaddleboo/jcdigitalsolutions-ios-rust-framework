#include <framework.hpp>

int main() {
  const framework::AbiVersion version = framework::AbiVersion::current();
  return version.major == 1 && version.minor >= 1 ? 0 : 1;
}
