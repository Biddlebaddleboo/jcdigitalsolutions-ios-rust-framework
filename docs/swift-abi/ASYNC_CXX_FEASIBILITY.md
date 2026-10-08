# Swift Async C++ Header Feasibility Proof

## Scope and result

This compiler proof asks whether the active Apple Swift compiler exposes public `async -> String` and `async throws -> String` declarations through its generated C++ interoperability header. It also emits a synchronous `String` control, inspects the compiler-emitted Swift symbols and host linkage, and stops before any C++ or Rust async caller is attempted if the generated C++ API omits the async declarations.

On Xcode 26.6 / Swift 6.3.3, header generation succeeded without compiler diagnostics. The synchronous control was exposed as a C++ `swift::String` wrapper. The header marked both async declarations unavailable in C++. The compiler still emitted Swift symbols for both functions, but a symbol alone is not a supported C++ API or sufficient evidence to reconstruct an async ABI.

Per C4, the C++/Rust async call-path probe stopped at that omission. No callback ABI, fixed-width C adapter, completion behavior, exactly-once behavior, Swift error path, String ownership path, cancellation support, or executor/task contract was guessed or claimed. Device and simulator C++/Rust caller objects were not built because no public generated async entry was available.

## Toolchain and SDK inventory

Recorded on macOS `26.6.2` (build `25G83`) with host `x86_64-apple-darwin`:

| Tool or target | Version or path |
|---|---|
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift | `swift-driver version: 1.148.6 Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Swift host target | `x86_64-apple-macosx26.0` |
| Apple Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)`, target `x86_64-apple-darwin25.6.0` |
| Rust | `rustc 1.94.1 (e408947bf 2026-03-25)`, commit `e408947bfd200af42db322daf0fadfe7e26d3bd1`, LLVM `21.1.8`, host `x86_64-apple-darwin` |
| macOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk`, version `26.5` |
| iPhoneOS SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk`, version `26.5` |
| iPhoneSimulator SDK | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk`, version `26.5` |
| Installed Rust Apple targets | `aarch64-apple-ios`, `aarch64-apple-ios-sim` |

The repository's required baseline is Xcode `27.x`; this proof is evidence for the installed Xcode 26.6 and SDK 26.5 only. The active public compiler help lists `-cxx-interoperability-mode`; `swiftc -frontend -help` lists `-emit-clang-header-path`. No private compiler option was used.

## Temporary declarations and compiler invocation

The shell proof writes this input under an external `mktemp` directory; it does not add a `.swift` source file to the checkout:

```swift
public func syncString(_ value:String)->String{value}
public func asyncString(_ value:String) async -> String{value}
public func asyncThrowsString(_ value:String) async throws -> String{value}
```

The host command was:

```sh
xcrun swiftc -parse-as-library -module-name SwiftAsyncProof -cxx-interoperability-mode=default -emit-clang-header-path "$host_dir/SwiftAsyncProof.h" -target x86_64-apple-macosx26.0 -emit-library -o "$host_dir/libSwiftAsyncProof.dylib" "$probe_dir/Oracle.swift"
```

It exited successfully. Captured compiler stdout and stderr were both zero bytes. The generated public C++ wrapper line for the synchronous control was:

```cpp
SWIFT_INLINE_THUNK swift::String syncString(const swift::String& value) noexcept SWIFT_SYMBOL("s:15SwiftAsyncProof10syncStringyS2SF") SWIFT_WARN_UNUSED_RESULT {
```

The exact generated omission comments were:

```cpp
// Unavailable in C++: Swift global function 'asyncString(_:)'.
// Unavailable in C++: Swift global function 'asyncThrowsString(_:)'.
```

There was no compiler diagnostic for the async declarations; they were explicitly marked unavailable in the successfully generated header. The script treats a future generated callable async declaration as a scope change and stops with an error so the bounded call-path proof can be extended deliberately.

## Compiler-emitted symbols and runtime observations

`nm -g` on the host Swift library reported:

```text
_$s15SwiftAsyncProof10syncStringyS2SF
_$s15SwiftAsyncProof11asyncStringyS2SYaF
_$s15SwiftAsyncProof17asyncThrowsStringyS2SYaKF
```

The async symbols confirm that the Swift compiler emitted Swift entry symbols. They do not establish C++ availability, a stable callable C ABI, async-context layout, continuation shape, result/error placement, or executor behavior. `nm -u` on this minimal Swift library reported `_swift_bridgeObjectRetain`; it reported no `swift_task`, `swift_continuation`, or `swift_async` undefined symbol. Since the declarations were never invoked, this is not evidence that an arbitrary Swift async operation needs no task/executor support.

`otool -L` on the Swift library listed these non-self dependencies:

```text
/usr/lib/libc++.1.dylib
/usr/lib/libSystem.B.dylib
/usr/lib/swift/libswiftCore.dylib
```

A temporary C `dlopen`/`dlclose` executable loaded the Swift library without invoking any Swift function. With `DYLD_PRINT_LIBRARIES=1`, the host loader reported one path load each for `/usr/lib/swift/libswiftCore.dylib`, `/usr/lib/swift/libswift_Concurrency.dylib`, and `/usr/lib/libc++.1.dylib`. This records the Swift library's host load closure only; it does not measure a C++ caller's linkage or load behavior. No C++ shim or executable was built because the async functions have no generated C++ entry.

## Device/simulator and unperformed call-path checks

The device and simulator SDKs were inventoried, but no iOS Swift/C++/Rust caller objects were built. C4 only calls for matching target caller objects if the generated public async API can be represented; it could not be represented here, so the call-path probe stopped at header generation.

Accordingly, this proof does not report a host success/error call, completion count, Swift `Error` conversion, String wrapper ownership/destruction, cancellation, Rust `#![no_std]` caller, target compile/link result, or App Store packaging. It does not infer executor requirements or ABI details from the async mangled symbols, and it does not use `swiftasynccall`, handwritten assembly, private metadata, or a guessed thunk signature as a fallback.

## Reproduction and validation

Run the proof on macOS with Xcode and Rust installed:

```sh
sh interop/swift-abi-core/tests/check-swift-async-cxx.sh
```

The script generates the Swift oracle, C++ header, Swift library, C loader, and inspection files under an external temporary directory and removes them on exit. Set `KEEP_SWIFT_ASYNC_PROBE=1` to retain the temporary evidence outside the checkout.

Focused validation:

```sh
sh interop/swift-abi-core/tests/check-swift-async-cxx.sh  # passed; async C++ omissions reproduced
sh -n interop/swift-abi-core/tests/check-swift-async-cxx.sh
cargo fmt --all -- --check
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
```

This result is compiler feasibility evidence for the exact toolchain above. It does not establish a production Swift async ABI, a Rust async adapter, cancellation or error semantics, Translation support, a stable cross-toolchain C++ ABI, or an iOS packaging path. A later async implementation must use a compiler-derived and supported public lowering rather than the emitted symbol names alone.
