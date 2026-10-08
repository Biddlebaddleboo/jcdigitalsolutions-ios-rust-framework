# Swift String C++ Interoperability Proof

## Scope and result

This proof tests whether the installed Apple Swift compiler can generate a C++ API for a temporary public Swift `String` round-trip, and whether a C++ shim can expose that operation to Rust through only fixed-width pointer-and-length arguments. It adds no production ABI API, Swift value storage, runtime metadata parser, Translation code, or shipping Swift/C++ source.

The proof passed with Xcode 26.6 / Swift 6.3.3. Host execution preserved exact UTF-8 bytes for all six fixtures, including embedded NUL. The same Swift oracle, generated C++ API, C++ shim, and Rust C-boundary caller compiled to objects for iOS device and simulator. Those target checks are compile-only; there was no target link, simulator execution, device execution, or app archive.

This is not a recommendation or claim that Swift C++ interoperability is a permanent production backend.

## Toolchain and SDK inventory

Recorded on macOS 26.6.2, host `x86_64-apple-darwin`:

| Tool | Version or path |
|---|---|
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift | `swift-driver version: 1.148.6 Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Swift host target | `x86_64-apple-macosx26.0` |
| Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)` |
| Rust | `rustc 1.94.1 (e408947bf 2026-03-25)`, LLVM 21.1.8 |
| macOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk`, version `26.5` |
| iPhoneOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk`, version `26.5` |
| iPhoneSimulator SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk`, version `26.5` |
| Swift C++ bridging headers | `$SWIFT_TOOLCHAIN_ROOT/usr/include/swift/bridging` |

The compiler reports `/usr/lib/swift` as a host runtime path. On this system `libswiftCore.dylib` is provided by the dyld shared cache, not as a standalone file at that path; the proof links and loads the runtime by that reported install path.

Xcode 26.6 is below the Xcode 27.x plan baseline. Results are specific to this installed toolchain and SDK.

## Compiler-authored input and generated API

The script writes only this temporary Swift oracle to its external `mktemp` directory:

```swift
public func makeString(_ bytes:[UInt8])->String{String(decoding:bytes,as:UTF8.self)}
public func roundTrip(_ value:String)->String{value}
```

The `[UInt8]` helper is needed for the required embedded-NUL fixture. The current compiler's generated C++ `std::string` constructor for `swift::String` calls `str.c_str()`, which would stop at NUL. The shim instead uses the generated `swift::Array<uint8_t>` API to pass the exact bytes to `makeString`; the public `roundTrip(_:)` entry accepts and returns `String` as required. No Swift String storage is constructed or inspected by Rust or handwritten C++.

The supported compiler invocation on the host was:

```sh
xcrun swiftc -parse-as-library -module-name SwiftStringProof -cxx-interoperability-mode=default -emit-clang-header-path "$probe_dir/host/SwiftStringProof.h" -target x86_64-apple-macosx26.0 -emit-library -o "$probe_dir/host/libSwiftStringProof.dylib" "$probe_dir/Oracle.swift"
```

`-cxx-interoperability-mode=default` is listed by `xcrun swiftc --help`; `-emit-clang-header-path` is listed by `xcrun swiftc -frontend -help` as emitting an Objective-C and C++ header. Header generation from this Swift test module succeeded; no private compiler flags, Apple SDK build metadata, or manually guessed Swift ABI signature was used.

The generated public C++ surface was:

```cpp
swift::String makeString(const swift::Array<uint8_t>& bytes) noexcept;
swift::String roundTrip(const swift::String& value) noexcept;
```

The generated header maps these declarations to these compiler-emitted Swift symbols:

```text
_$s16SwiftStringProof04makeB0ySSSays5UInt8VGF
_$s16SwiftStringProof9roundTripyS2SF
```

`nm` reported both as definitions in the host, device, and simulator Swift objects, and as unresolved references in the matching C++ shim objects. The generated C++ header wraps Swift values using compiler-emitted `swift::String` and `swift::Array<uint8_t>` C++ types. These types and their layout do not cross the Rust boundary.

The host shim's `nm -u` output also contains Swift standard-library references used by the generated wrapper and String UTF-8 overlay, including `_$sS2ayxGycfC` (Array initialization), `_$sSa6appendyyxnF` (Array append), `_$sSaMa` (Array metadata), `_$ss5UInt8VN` (UInt8 metadata), `_$sSSMa` (String metadata), and `_$sSS4utf8SS8UTF8ViewVvg` (String UTF-8 view). The host Swift dylib and Rust executable resolve these through the active Swift runtime; the proof does not define these symbols itself.

## C ABI and UTF-8 fixture result

The temporary C++ shim exports only this C ABI shape:

```c
int32_t framework_swift_string_round_trip(
    const uint8_t *input,
    uint64_t input_len,
    uint8_t *output,
    uint64_t output_capacity,
    uint64_t *output_len);
```

Rust calls this function with no C++/Swift types. The C++ side constructs a Swift byte array using generated C++ wrappers, calls the generated `makeString` and `roundTrip` APIs, converts the returned value through the generated `String.UTF8View` overlay, and copies the resulting `std::string` bytes to the output pointer. Output length is explicit. The shim catches C++ exceptions so none cross the C boundary.

The host Rust executable asserted both output length and exact byte equality:

| Fixture | UTF-8 bytes (hex) | Length |
|---|---|---:|
| Empty | — | 0 |
| ASCII `ASCII` | `41 53 43 49 49` | 5 |
| Embedded NUL `A\0B` | `41 00 42` | 3 |
| BMP `é` | `C3 A9` | 2 |
| Combining sequence `e` + U+0301 | `65 CC 81` | 3 |
| Multi-scalar emoji `👩‍💻` | `F0 9F 91 A9 E2 80 8D F0 9F 92 BB` | 11 |

All six cases passed. No normalization or lossy UTF-8 conversion occurred for these fixtures.

## Generated value ownership evidence

The generated header's `swift::String` copy constructor uses the String metadata's `initializeWithCopy` value witness. Its destructor obtains the same metadata's value witness table and calls `destroy`. The C++ shim explicitly copies the input String, uses the copy in the generated round-trip call, and leaves the input, copy, and returned String in scope for normal destruction on each call.

With `clang++ -O0 -S -emit-llvm`, the shim IR contains the generated C++ String copy-constructor call (`_ZN5swift6StringC1ERKS0_`) and String destructor calls (`_ZN5swift6StringD1Ev`). The host executable ran all six cases. This is value-witness copy/destroy evidence, not a direct `swift_retain`/`swift_release` class-reference proof and not a measured allocation/refcount count. No `_StringGuts` layout was read, encoded, or modified.

## Host runtime and linkage

The host Rust executable directly links the temporary Swift oracle dylib, `libc++`, and one `libswiftCore.dylib`. `otool -L` reports the Swift runtime dependency as `/usr/lib/swift/libswiftCore.dylib`; `DYLD_PRINT_LIBRARIES=1` recorded exactly one load of that path for `libswiftCore.dylib`. Other normal runtime dependencies include `libc++`, `libc++abi`, `libSystem`, and Swift support overlays such as `libswiftCoreFoundation.dylib`, `libswiftCoreAudio.dylib`, `libswiftCoreMedia.dylib`, `libswiftCoreMIDI.dylib`, `libswiftCoreImage.dylib`, and `libswiftCoreLocation.dylib`. This is not a minimal-linkage or binary-size claim.

The default `swift-abi-core` feature path remains free of Swift runtime imports/linkage per the separately rerun [C1 ownership proof](OWNERSHIP_RUNTIME_PROOF.md) and its `check-retained-ownership.sh` gate. That C1 result does not imply this opt-in C++ String probe has no Swift runtime cost.

## Device and simulator compile-only evidence

The target commands used the SDK paths above and these triples:

| Probe | Swift and C++ target | Rust target |
|---|---|---|
| Device | `arm64-apple-ios17.0`, iPhoneOS SDK `26.5` | `aarch64-apple-ios` |
| Simulator | `arm64-apple-ios17.0-simulator`, iPhoneSimulator SDK `26.5` | `aarch64-apple-ios-sim` |

For each target, `swiftc` emitted the compiler-generated header and Swift object; Apple Clang compiled the C++ shim against that matching target header; `rustc --crate-type lib --emit=obj` compiled a `#![no_std]` caller that references `framework_swift_string_round_trip`. Symbol inspection confirmed the C++ objects reference the corresponding compiler-emitted Swift symbols and the Rust objects reference the fixed-width C ABI function. The objects were not linked together for iOS, and no target runtime call or availability claim follows.

Exact device compile commands (paths are from the inventory above; `$probe_dir` and `$SWIFT_TOOLCHAIN_ROOT` are temporary script variables):

```sh
xcrun swiftc -parse-as-library -module-name SwiftStringProof -cxx-interoperability-mode=default -emit-clang-header-path "$probe_dir/device/SwiftStringProof.h" -target arm64-apple-ios17.0 -sdk "$device_sdk" -emit-object -o "$probe_dir/device/oracle.o" "$probe_dir/Oracle.swift"
xcrun --sdk iphoneos clang++ -target arm64-apple-ios17.0 -isysroot "$device_sdk" -std=c++17 -O0 -I "$SWIFT_TOOLCHAIN_ROOT/usr/include" -I "$probe_dir/device" -c -o "$probe_dir/device/shim.o" "$probe_dir/shim.cpp"
rustc --edition=2021 --crate-type lib --target aarch64-apple-ios --emit=obj "$probe_dir/target-caller.rs" -o "$probe_dir/device/rust-caller.o"
```

The simulator commands use the same flags with `--sdk iphonesimulator`, `-target arm64-apple-ios17.0-simulator`, `-sdk "$simulator_sdk"`, `-isysroot "$simulator_sdk"`, `-I "$probe_dir/simulator"`, and `--target aarch64-apple-ios-sim`; outputs go under `$probe_dir/simulator`.

## Reproduction and validation

Run the complete proof on macOS with Xcode and Rust iOS targets installed:

```sh
sh interop/swift-abi-core/tests/check-swift-string-cxx.sh
```

The script uses only temporary Swift source, generated headers, C++ source, IR, assembly, objects, libraries, and executables under an external `mktemp` directory. Its trap removes that directory. The observed host result was six exact fixture passes and one `libswiftCore.dylib` load. Device and simulator Swift, C++, and Rust compilation both passed for the triples above.

The host Rust link used `-C link-arg="$probe_dir/host/shim.o"`, `-C link-arg="$probe_dir/host/libSwiftStringProof.dylib"`, `-C link-arg=-lc++`, `-C link-arg=-L/usr/lib/swift`, `-C link-arg=-lswiftCore`, `-C link-arg=-Wl,-rpath,/usr/lib/swift`, `-C link-arg=-Wl,-rpath,"$probe_dir/host"`, and `-C link-arg=-mmacosx-version-min=26.0`, then ran `DYLD_PRINT_LIBRARIES=1 "$probe_dir/host/rust-host-probe"`. Explicit Swift and C++ runtime links are required by this opt-in proof; the default `swift-abi-core` feature path remains separately runtime-free.

Focused repository validation passed:

```sh
sh -n interop/swift-abi-core/tests/check-swift-string-cxx.sh
cargo fmt --all -- --check
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
```

The zero-Swift-source check confirms no `.swift` source was added to the checkout. The existing default-feature linkage check was also rerun:

```sh
sh interop/swift-abi-core/tests/check-retained-ownership.sh
```

It passed, including the default-feature host executable audit with no `libswiftCore.dylib` linkage or `_swift_retain` / `_swift_release` imports.

## Limits and follow-up decision

- Swift C++ interoperability and its generated API are compiler-version/toolchain dependent; the generated header also uses the active toolchain's `swiftToCxx` standard-library overlay. This probe does not establish a supported cross-version C++ ABI contract.
- The host proof adds Swift runtime and C++ runtime dependencies. No binary-size measurement, Release app integration, iOS link, Xcode archive, signing, installation, packaging, or App Store review was performed.
- Device and simulator results prove compilation only at the stated iOS 17.0 target and Xcode 26.5 SDKs. No minimum deployment target was established.
- No C++ wrappers are retained in the repository; no compiler-generated header is checked in.
- The byte-array helper and identity `roundTrip` are synthetic. No Apple framework declaration, Translation operation, async behavior, generic Swift ABI policy, or reusable Swift String runtime is proven.
- A later backend decision must assess Swift C++ interoperability stability, headers and toolchain dependency, runtime and C++ library packaging, and App Store distribution before product use.
