# Synchronous Swift Thunk Feasibility

## Scope

One scalar-only compiler-oracle proof for a C-ABI thunk that calls a Swift-ABI function

The probe does not implement a reusable thunk generator, opaque value storage, or an Apple framework API call

## Toolchain and targets

| Item | Value |
|---|---|
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift | `Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)` |
| Host target | `x86_64-apple-macosx26.0` |
| Device probe | `arm64-apple-ios17.0`, iPhoneOS SDK `26.5` |
| Simulator probe | `arm64-apple-ios17.0-simulator`, iPhoneSimulator SDK `26.5` |

The iOS 17.0 target is an explicit compiler probe, not a framework deployment floor. The repo has no shared iOS deployment target. Xcode 26.6 also remains below the Xcode 27.x plan baseline

## Temporary oracle

The only Swift input was `/tmp/jcdig-swift-abi-phase1/Oracle.swift`; it was not part of the checkout:

```swift
public func add(_ lhs: Int32, _ rhs: Int32) -> Int32 { lhs &+ rhs }
```

Swift 6.3.3 emitted this host, device, and simulator LLVM IR declaration:

```llvm
define swiftcc i32 @"$s13SwiftAbiProof3addys5Int32VAD_ADtF"(i32 %0, i32 %1)
```

`nm -g` on the compiler-produced device and simulator objects reported the definition as `_$s13SwiftAbiProof3addys5Int32VAD_ADtF`

## Clang thunk proof

The temporary Clang source declared that exact object symbol and exact IR-derived signature, then exported a normal C-ABI wrapper:

```c
#include <stdint.h>
extern int32_t swift_abi_add(int32_t, int32_t) __asm__("_$s13SwiftAbiProof3addys5Int32VAD_ADtF") __attribute__((swiftcall));
__attribute__((visibility("default"))) int32_t framework_swift_abi_add(int32_t lhs, int32_t rhs) { return swift_abi_add(lhs, rhs); }
```

The separate host caller used:

```c
extern int32_t framework_swift_abi_add(int32_t, int32_t);
int main(void) { return framework_swift_abi_add(19, 23) == 42 ? 0 : 1; }
```

Clang 21 emitted a C-convention wrapper plus a `swiftcc` call:

```llvm
define i32 @framework_swift_abi_add(i32 noundef %0, i32 noundef %1)
  %3 = tail call swiftcc i32 @"\01_$s13SwiftAbiProof3addys5Int32VAD_ADtF"(i32 noundef %0, i32 noundef %1)
```

The host proof linked this object with the Swift compiler object and a separate C caller. The C caller invoked `framework_swift_abi_add(19, 23)` and the executable exited `0`

## Rust caller proof

[`check-scalar-swiftcall.sh`](../../interop/swift-abi-core/tests/check-scalar-swiftcall.sh) makes the same fixture reproducible. It writes Swift, C, and Rust inputs to a `mktemp` directory outside the checkout and removes that directory on exit. The script reads the Swift symbol from `nm`, checks the Swift LLVM IR for `swiftcc i32(i32, i32)`, then emits the Clang shim declaration from that compiler output

The host Rust executable calls `framework_swift_abi_add` through `extern "C"`; the shim calls the compiler-emitted function through `swiftcall`. The executable linked and ran with exit `0`

For device and simulator, the script compiles Swift, Clang, and `no_std` Rust objects. It checks the compiler-derived symbol across both objects and the Swift call in Clang IR/assembly. These are compile-only results; no target executable link or run occurs

Run on macOS with Xcode and the Rust iOS targets installed:

```sh
./interop/swift-abi-core/tests/check-scalar-swiftcall.sh
```

## Device and simulator compile proof

The same temporary Swift and Clang inputs compiled for both iOS targets. `nm -g` showed a defined Swift symbol in the Swift object, an undefined reference to that same symbol in the Clang thunk object, and the exported `_framework_swift_abi_add` wrapper

For both arm64 targets, optimized Clang assembly reduced the scalar wrapper body to one direct branch:

```asm
b _$s13SwiftAbiProof3addys5Int32VAD_ADtF
```

The Swift compiler IR and Clang thunk IR used the same direct `i32 (i32, i32)` signature on host, device, and simulator. The host executable confirms runtime behavior on macOS; device and simulator results are compile/object evidence only

## Reproduction commands

The temporary Swift source above was compiled with:

```sh
xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProof -o /tmp/jcdig-swift-abi-phase1/oracle-host.ll /tmp/jcdig-swift-abi-phase1/Oracle.swift
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProof -o /tmp/jcdig-swift-abi-phase1/oracle-host.o /tmp/jcdig-swift-abi-phase1/Oracle.swift
xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProof -target arm64-apple-ios17.0 -sdk /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk -o /tmp/jcdig-swift-abi-phase1/oracle-device.ll /tmp/jcdig-swift-abi-phase1/Oracle.swift
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProof -target arm64-apple-ios17.0 -sdk /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk -o /tmp/jcdig-swift-abi-phase1/oracle-device.o /tmp/jcdig-swift-abi-phase1/Oracle.swift
xcrun swiftc -parse-as-library -emit-ir -O -module-name SwiftAbiProof -target arm64-apple-ios17.0-simulator -sdk /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk -o /tmp/jcdig-swift-abi-phase1/oracle-simulator.ll /tmp/jcdig-swift-abi-phase1/Oracle.swift
xcrun swiftc -parse-as-library -emit-object -O -module-name SwiftAbiProof -target arm64-apple-ios17.0-simulator -sdk /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk -o /tmp/jcdig-swift-abi-phase1/oracle-simulator.o /tmp/jcdig-swift-abi-phase1/Oracle.swift
```

The Clang thunk was compiled with the matching target and SDK:

```sh
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk -S -emit-llvm -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-device.ll /tmp/jcdig-swift-abi-phase1/Thunk.c
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk -S -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-device.s /tmp/jcdig-swift-abi-phase1/Thunk.c
xcrun --sdk iphoneos clang -target arm64-apple-ios17.0 -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk -c -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-device.o /tmp/jcdig-swift-abi-phase1/Thunk.c
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk -S -emit-llvm -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-simulator.ll /tmp/jcdig-swift-abi-phase1/Thunk.c
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk -S -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-simulator.s /tmp/jcdig-swift-abi-phase1/Thunk.c
xcrun --sdk iphonesimulator clang -target arm64-apple-ios17.0-simulator -isysroot /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk -c -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-simulator.o /tmp/jcdig-swift-abi-phase1/Thunk.c
```

The host Clang objects and proof executable used:

```sh
clang -S -emit-llvm -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-host.ll /tmp/jcdig-swift-abi-phase1/Thunk.c
clang -c -O2 -o /tmp/jcdig-swift-abi-phase1/thunk-host.o /tmp/jcdig-swift-abi-phase1/Thunk.c
clang -c -O2 -o /tmp/jcdig-swift-abi-phase1/main-host.o /tmp/jcdig-swift-abi-phase1/Main.c
clang /tmp/jcdig-swift-abi-phase1/main-host.o /tmp/jcdig-swift-abi-phase1/thunk-host.o /tmp/jcdig-swift-abi-phase1/oracle-host.o -o /tmp/jcdig-swift-abi-phase1/proof-host
/tmp/jcdig-swift-abi-phase1/proof-host
```

## Limits

- Proof covers only two direct `Int32` arguments and one direct `Int32` result
- The source is a synthetic fixture, not a declaration from an Apple SDK
- The host Rust caller links and runs; no framework Rust archive or application archive link was part of this proof
- No device or simulator executable link or runtime call was part of this proof
- No Swift class ownership, metadata, value witnesses, resilient value, indirect result, generic argument, context, throw, or async shape was part of this proof
- No general API symbol selection or symbol-version policy follows from this fixture
- No ordinary iOS backend rule or framework API support claim changes here

This result establishes that the installed Apple Clang can express this exact SwiftCC shape behind a C-ABI wrapper. Each real Apple API still needs compiler-oracle lowering, public-symbol provenance, target availability, and ownership proof before a thunk can call it
