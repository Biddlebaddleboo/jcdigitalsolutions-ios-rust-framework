# Swift `Optional<String>` C++ Interoperability Proof

## Scope and result

This proof checks whether the active Apple Swift compiler can expose `String?` through its generated C++ interoperability header, then lets a Rust caller reach that compiler-authored path through only fixed-width C fields. The Swift oracle, generated header, C++ shim, IR, objects, library, executable, and temporary build files live outside the checkout. This adds no production ABI module, shipping Swift source, C++ wrapper, Optional layout, or runtime metadata parser.

The proof passed on Xcode 26.6 / Swift 6.3.3. Host execution distinguished `none` from `some` with an empty string and preserved exact UTF-8 bytes for all remaining fixtures, including embedded NUL. Swift, C++, and `#![no_std]` Rust caller objects also compiled for iOS device and simulator. Those target checks are compile-only: no target link, simulator or device execution, app archive, signing, installation, or App Store packaging was attempted.

The C++ API is compiler/toolchain dependent. This is not a production Optional adapter or a claim of a stable cross-toolchain C++ ABI.

## Toolchain and SDK inventory

Recorded on macOS 26.6.2 (build 25G83), host `x86_64-apple-darwin`:

| Tool | Version or path |
|---|---|
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift | `swift-driver version: 1.148.6 Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Swift host target | `x86_64-apple-macosx26.0` |
| Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)` |
| Rust | `rustc 1.94.1 (e408947bf 2026-03-25)`, LLVM 21.1.8, host `x86_64-apple-darwin` |
| macOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk`, version `26.5` |
| iPhoneOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk`, version `26.5` |
| iPhoneSimulator SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk`, version `26.5` |
| Swift C++ bridging headers | `$SWIFT_TOOLCHAIN_ROOT/usr/include/swift/bridging` |
| Rust targets installed | `aarch64-apple-ios`, `aarch64-apple-ios-sim` |

Xcode 26.6 is below the plan's Xcode 27.x baseline. The result is evidence for this installed toolchain and SDK only. The active compiler's public `swiftc --help` lists `-cxx-interoperability-mode`; `swiftc -frontend -help` lists `-emit-clang-header-path`. The proof uses those documented flags and no private compiler option or guessed Swift ABI signature.

## Compiler-authored input and generated API

The script writes only this temporary Swift oracle under an external `mktemp` directory:

```swift
public func makeOptional(_ bytes:[UInt8],_ present:Bool)->String?{guard present else{return nil};return String(decoding:bytes,as:UTF8.self)}
public func roundTrip(_ value:String?)->String?{value}
```

The `[UInt8]` helper preserves the required embedded-NUL fixture. `makeOptional` supplies both `none` and `some`, including `some` with zero bytes. The public round-trip function accepts and returns `String?`. C++ constructs its input through the generated `swift::Array<uint8_t>` and generated wrapper, then receives a generated `swift::Optional<swift::String>`; neither layout is read, constructed manually, or passed to Rust.

The host invocation was:

```sh
xcrun swiftc -parse-as-library -module-name SwiftOptionalProof -cxx-interoperability-mode=default -emit-clang-header-path "$probe_dir/host/SwiftOptionalProof.h" -target x86_64-apple-macosx26.0 -emit-library -o "$probe_dir/host/libSwiftOptionalProof.dylib" "$probe_dir/Oracle.swift"
```

The generated public C++ declarations are:

```cpp
swift::Optional<swift::String> makeOptional(const swift::Array<uint8_t>& bytes, bool present) noexcept;
swift::Optional<swift::String> roundTrip(const swift::Optional<swift::String>& value) noexcept;
```

The generated wrapper exposes `swift::Optional<swift::String>::isNone()` and `getSome()` plus compiler-emitted `none`/`some` constructors. It does not expose Rust to any C++ wrapper storage. The host Swift library defines these compiler-emitted functions and the host C++ object references them:

```text
_$s18SwiftOptionalProof04makeB0ySSSgSays5UInt8VG_SbtF
_$s18SwiftOptionalProof9roundTripySSSgACF
```

Device and simulator objects emitted the same two function symbols as the host oracle. Header generation and all target compilation succeeded on the first compiler probe; there were no failed Swift, C++, or Rust compiler probes, no fallback invocation, and no unsupported Optional mapping diagnostic.

## Fixed-width C boundary and fixture results

The temporary C++ shim exports only this C ABI:

```c
int32_t framework_swift_optional_string_round_trip(
    uint32_t input_has_value,
    const uint8_t *input,
    uint64_t input_len,
    uint8_t *output,
    uint64_t output_capacity,
    uint32_t *output_has_value,
    uint64_t *output_len);
```

`input_has_value` and `output_has_value` are `0` for none and `1` for some. Presence is independent of length, so `some` with empty text has state `1` and length `0`; `none` has state `0` and length `0`. The status is `0` on success, `1` for missing required output-state/output-length pointers, `2` for an input state other than `0` or `1`, `3` for a null non-empty input, `4` for a null output with nonzero capacity, `5` for insufficient output capacity, and `6` for a caught C++ exception. No C++ or Swift layout crosses this boundary. The C++ side catches exceptions instead of letting them cross the C ABI.

The host Rust executable asserted status, optional presence, output length, and exact bytes:

| Case | State | UTF-8 bytes (hex) | Length |
|---|---:|---|---:|
| `none` | 0 | — | 0 |
| `some` empty | 1 | — | 0 |
| ASCII `ASCII` | 1 | `41 53 43 49 49` | 5 |
| Embedded NUL `A\0B` | 1 | `41 00 42` | 3 |
| BMP `é` | 1 | `C3 A9` | 2 |
| Combining sequence `e` + U+0301 | 1 | `65 CC 81` | 3 |
| Multi-scalar emoji `👩‍💻` | 1 | `F0 9F 91 A9 E2 80 8D F0 9F 92 BB` | 11 |

All seven fixtures passed. The Rust caller also supplied invalid presence `2` and verified the documented error status `2`. No Unicode normalization or lossy conversion occurred for these fixtures.

## Generated ownership evidence and object inspection

The compiler-generated `swift::Optional<T>` wrapper obtains Optional metadata and its value-witness table. Its copy constructor calls `initializeWithCopy`; its destructor calls `destroy`. The generated `swift::String` wrapper likewise uses compiler-provided String metadata value witnesses for copy and destruction. The temporary shim deliberately copies both `Optional<String>` and the extracted `String`; its host execution then destroys them through generated wrappers for all seven cases.

At `clang++ -O0 -S -emit-llvm`, host C++ IR contains exercised `swift::Optional<swift::String>` copy-constructor and destructor calls and `swift::String` copy-constructor and destructor calls. In the Optional copy implementation, the generated IR loads the copy function from the Optional value-witness table and invokes it; the destructor loads and invokes the destroy witness. `nm -u` on the host C++ object confirms references to the compiler-emitted `makeOptional` and `roundTrip` symbols, Optional metadata `_$sSqMa`, String metadata `_$sSSMa`, and Swift standard-library operations. The IR also shows the generated C++ wrappers invoke the two compiler-authored Swift functions. This is generated value-witness copy/destroy evidence, not a guessed Optional layout, a direct class retain/release proof, or a measured allocation/refcount count.

The host C++ compile command was:

```sh
xcrun clang++ -target x86_64-apple-macosx26.0 -isysroot "$host_sdk" -std=c++17 -O0 -I "$SWIFT_TOOLCHAIN_ROOT/usr/include" -I "$probe_dir/host" -S -emit-llvm -o "$probe_dir/host/shim.ll" "$probe_dir/shim.cpp"
```

The object check confirms only fixed-width C function arguments/returns on the Rust side. The C++ object imports Swift metadata and overlay operations because it is the opt-in compiler-generated value boundary.

## Host runtime and default-feature linkage

The Rust host executable directly links the temporary Swift oracle library, `libc++`, and one Swift core runtime. `otool -L` reports exactly one `libswiftCore.dylib` link; `DYLD_PRINT_LIBRARIES=1` records exactly one load at `/usr/lib/swift/libswiftCore.dylib`. Other runtime/support dependencies may be present; this proof is not a minimal-linkage or binary-size claim.

The separately rerun `sh interop/swift-abi-core/tests/check-retained-ownership.sh` passed. Its default `swift-abi-core` feature-path audit confirmed no `libswiftCore.dylib` linkage and no `swift_retain`/`swift_release` imports. This Optional C++ proof is opt-in and does not change the default feature path.

## Device and simulator compile-only evidence

For each target, `swiftc` emitted the generated header and Swift object; Apple Clang compiled the C++ shim against that target header; and Rust compiled the `#![no_std]` caller with `--emit=obj`. `nm` confirmed that C++ objects reference both compiler-emitted Swift functions and Rust objects reference the fixed-width C ABI symbol. The objects were not linked together and were not executed.

| Probe | Swift and C++ target | Rust target | SDK |
|---|---|---|---|
| Device | `arm64-apple-ios17.0` | `aarch64-apple-ios` | iPhoneOS 26.5 |
| Simulator | `arm64-apple-ios17.0-simulator` | `aarch64-apple-ios-sim` | iPhoneSimulator 26.5 |

The target Swift command form was:

```sh
xcrun swiftc -parse-as-library -module-name SwiftOptionalProof -cxx-interoperability-mode=default -emit-clang-header-path "$target_dir/SwiftOptionalProof.h" -target "$target_triple" -sdk "$sdk_path" -emit-object -o "$target_dir/oracle.o" "$probe_dir/Oracle.swift"
```

The target C++ command form was:

```sh
xcrun --sdk "$sdk_name" clang++ -target "$target_triple" -isysroot "$sdk_path" -std=c++17 -O0 -I "$SWIFT_TOOLCHAIN_ROOT/usr/include" -I "$target_dir" -c -o "$target_dir/shim.o" "$probe_dir/shim.cpp"
```

Rust compiled with `rustc --edition=2021 --crate-type lib --target "$rust_target" --emit=obj "$probe_dir/target-caller.rs" -o "$target_dir/rust-caller.o"` for each target.

## Reproduction and limits

Run on macOS with Xcode and both Rust iOS targets installed:

```sh
sh interop/swift-abi-core/tests/check-swift-optional-cxx.sh
```

The script places every oracle and generated/build artifact under an external `mktemp` directory and removes it on exit. Set `KEEP_SWIFT_OPTIONAL_PROBE=1` only when retaining those external artifacts for inspection. It validates the generated API, Optional and String value-witness copy/destroy source, host IR/object references, host fixture execution, one Swift core runtime load/link, and device/simulator object compilation.

Focused repository validation:

```sh
sh interop/swift-abi-core/tests/check-swift-optional-cxx.sh  # passed
sh -n interop/swift-abi-core/tests/check-swift-optional-cxx.sh
cargo fmt --all -- --check  # passed after cherry-picking the root-owned formatting fix
cargo xtask docs-check  # passed
cargo xtask zero-swift-source  # passed
git diff --cached --check  # passed for the staged C3 changes
```

The first repository-wide format check reported an unrelated pre-existing format diff at the `d6a5a56` base:

```text
Diff in crates/framework-sharing/src/share.rs:48:
     pub fn new(item: ShareItem) -> Self {
-        Self { items: alloc::vec![item] }
+        Self {
+            items: alloc::vec![item],
+        }
```

The root owner supplied the fix as `a931f7608842c37a87e16d5b365bd12f8d35c722`; its patch was cherry-picked into this isolated branch as `1fb1f89`, after which `cargo fmt --all -- --check` passed. C3 does not modify `crates/framework-sharing/src/share.rs`.

No `.swift` source, generated header, C++ wrapper, or proof binary is retained in the checkout.

This result does not establish a stable cross-toolchain C++ ABI, production Swift Optional adapter, Translation readiness, binary-size benefit, minimum supported deployment target, linked iOS app, app archive, or App Store packaging. A later backend decision must reassess compiler/header compatibility, runtime and C++ library packaging, and the concrete public Apple API that requires `Optional<String>`.
