# Swift Async Clang Thunk Feasibility

## Scope and result

This compiler proof tests whether Apple Clang can describe and forward a compiler-derived Swift async entry for a minimal `async -> Int32` and `async throws -> Int32` API. The fixture also contains Swift-generated async callers so its task-frame, resume, and error paths can serve as an oracle.

Clang 21 accepts the public `swiftasynccall` and `swift_async_context` attributes on the host, iOS device, and iOS simulator targets. When the C declaration follows the exact parameter order emitted by Swift LLVM IR, Clang emits the same `swifttailcc` function type and a tail transfer to the compiler-emitted symbol. The ARM64 Clang object contains a direct branch to each Swift entry symbol. This establishes a bounded calling-convention lowering match, not a callable Rust async adapter.

The proof stops before invoking either entry. The compiler-generated Swift caller allocates and initializes a task context, installs a resume function, calls task-runtime entrypoints, and handles the success/error continuation. `swiftasynccall` only expresses the low-level call convention; the proof does not establish a public C/Rust contract for task creation, context layout/lifetime, resume ownership, executor behavior, or Swift error delivery. The existing Rust `OperationState<T,E>` completion/waker state does not provide that Swift task/runtime layer. No stable cross-toolchain ABI, Rust `Future` adapter, cancellation bridge, Translation support, or StoreKit support is claimed.

## Toolchain and targets

Recorded on macOS `26.6.2` (build `25G83`) with host target `x86_64-apple-macosx26.0`:

| Tool or target | Version or path |
|---|---|
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift | `swift-driver version: 1.148.6 Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Apple Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)` |
| Host target / macOS SDK | `x86_64-apple-macosx26.0`; macOS SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk` |
| iOS device target / SDK | `arm64-apple-ios17.0`; iPhoneOS SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk` |
| iOS simulator target / SDK | `arm64-apple-ios17.0-simulator`; iPhoneSimulator SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk` |

Xcode 26.6 / SDK 26.5 is the installed proof environment and remains below the repository's Xcode `27.x` baseline. iOS `17.0` is a compile target for this ABI probe, not a framework deployment floor.

## Temporary Swift fixture and lowering

The script writes this source under an external `mktemp` directory and removes it on exit. No `.swift` file is added to the checkout:

```swift
public enum OracleFailure: Error { case failed }
public func asyncIdentity(_ value: Int32) async -> Int32 { value }
public func asyncThrowsIdentity(_ value: Int32) async throws -> Int32 { if value < 0 { throw OracleFailure.failed }; return value }
public func asyncCaller(_ value: Int32) async -> Int32 { await asyncIdentity(value) }
public func asyncThrowsCaller(_ value: Int32) async -> Int32 { do { return try await asyncThrowsIdentity(value) } catch { return -1 } }
```

Swift 6.3.3 SILGen emitted these high-level signatures:

```text
$@convention(thin) @async (Int32) -> Int32
$@convention(thin) @async (Int32) -> (Int32, @error any Error)
```

The compiler-emitted host, device, and simulator LLVM IR entry signatures were identical:

```llvm
define swifttailcc void @"$s20SwiftAsyncThunkProof13asyncIdentityys5Int32VADYaF"(ptr swiftasync %0, i32 %1)
define swifttailcc void @"$s20SwiftAsyncThunkProof19asyncThrowsIdentityys5Int32VADYaKF"(ptr swiftasync %0, i32 %1)
```

The compiler-emitted object symbols were:

```text
_$s20SwiftAsyncThunkProof13asyncIdentityys5Int32VADYaF
_$s20SwiftAsyncThunkProof19asyncThrowsIdentityys5Int32VADYaKF
```

The direct result is not a normal LLVM return value: both entries return `void` and take a special async-context pointer followed by the source `Int32` argument. The SIL `@error` result for the throwing function is carried through its compiler-generated continuation path, not as a synchronous `swift_error_result` parameter.

## Clang declaration and ABI comparison

The Clang C declaration was emitted by the script from the exact Swift object symbols above. The `swift_async_context` parameter is first because that is the order in the compiler-generated LLVM function type; it is a low-level lowered declaration, not the source-level Swift parameter order:

```c
extern void swift_async_identity(void *context __attribute__((swift_async_context)), int32_t value) __attribute__((swiftasynccall)) __asm__("_$s20SwiftAsyncThunkProof13asyncIdentityys5Int32VADYaF");
extern void swift_async_throws_identity(void *context __attribute__((swift_async_context)), int32_t value) __attribute__((swiftasynccall)) __asm__("_$s20SwiftAsyncThunkProof19asyncThrowsIdentityys5Int32VADYaKF");
```

For each target, Clang 21 emitted this forwarding shape (the `noundef` parameter attributes are Clang annotations; the calling convention, parameter order, and `swiftasync` treatment match the Swift oracle):

```llvm
define swifttailcc void @clang_async_identity(ptr noundef swiftasync %0, i32 noundef %1)
musttail call swifttailcc void @"\01_$s20SwiftAsyncThunkProof13asyncIdentityys5Int32VADYaF"(ptr noundef swiftasync %0, i32 noundef %1)
```

The throwing entry produced the same argument/calling-convention shape and referenced its own compiler-emitted symbol. The host Clang assembly tail-jumped to each exact Swift symbol. On both ARM64 iOS targets, each Clang forwarding function compiled to a direct branch:

```asm
b _$s20SwiftAsyncThunkProof13asyncIdentityys5Int32VADYaF
b _$s20SwiftAsyncThunkProof19asyncThrowsIdentityys5Int32VADYaKF
```

For device and simulator, `nm` found each Swift symbol defined by the Swift object and undefined by the Clang object. Both objects compiled successfully; no target link or runtime call occurred. The host evidence is also compile/object evidence only for the async entry call. The [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html) describes `swiftasynccall` as a low-level Swift async convention requiring the correct formal arguments, with `void` return and optional `swift_async_context`; it does not define Swift task construction or resumption.

## Compiler-generated caller, task, and resume evidence

The Swift async callers provide more than a symbol-name oracle. In the emitted IR for this fixture, `asyncCaller` and `asyncThrowsCaller`:

- read the compiler-emitted async-function descriptor's context-frame size;
- call `swift_task_alloc`;
- store the parent async context and compiler-generated resume function into the allocated frame;
- tail-call the async entry with `ptr swiftasync` and the direct `i32` argument;
- resume through compiler-generated continuation functions and call `swift_task_dealloc`.

This Xcode 26.6 fixture emitted descriptor size fields `16` for `asyncIdentity` and `32` for `asyncThrowsIdentity`. These are compiler output for this fixture, not stable layouts or values for a Rust caller.

The compiler-generated `asyncCaller` continuation has this signature:

```llvm
define internal swifttailcc void @"$s20SwiftAsyncThunkProof11asyncCallerys5Int32VADYaFTQ0_"(ptr swiftasync %0, i32 %1)
```

The throwing caller's continuation has a separate Swift-context error argument:

```llvm
define internal swifttailcc void @"$s20SwiftAsyncThunkProof17asyncThrowsCallerys5Int32VADYaFTQ0_"(ptr swiftasync %0, i32 %1, ptr swiftself %2)
```

The throwing callee's generated resume code calls `swift_task_switch`; its completion path forwards either an `Int32` result with a null `swiftself` pointer or an error object through `swiftself`. The fixture's catch path later calls `swift_errorRelease`. These are observations of this compiler-generated fixture only, not a general Swift error ownership contract. The compiler output does not establish a supported public C/Rust entry or contract to allocate a valid task context, establish executor state, and assume responsibility for resume and error completion.

Clang's `swiftasynccall` supports the matching entry convention and guaranteed tail-call shape, but cannot derive or populate that runtime context. Therefore this proof does not invoke the forwarding functions and does not use private runtime APIs, descriptor decoding, guessed context layouts, or hand-written assembly to fabricate a caller.

## Runtime and linkage observations

`nm -u` on the host Swift dynamic library recorded compiler-generated references to:

```text
_swift_task_alloc
_swift_task_dealloc
_swift_task_switch
```

`otool -L` recorded these non-self dependencies:

```text
/usr/lib/libSystem.B.dylib
/usr/lib/swift/libswiftCore.dylib
/usr/lib/swift/libswift_Concurrency.dylib
```

These are imports/dependencies observed for the compiled fixture, not a declaration of requirements for every Swift async call or a supported C/Rust runtime API. The host library was not loaded and no Swift function was invoked.

## Reproduction and validation

Run on macOS with Xcode:

```sh
sh interop/swift-abi-core/tests/check-swift-async-thunk.sh
```

The script creates Swift and C probe inputs, SIL, LLVM IR, assembly, objects, a host dynamic library, and inspection output in an external temporary directory. Set `KEEP_SWIFT_ASYNC_THUNK_PROBE=1` to preserve that evidence outside the checkout.

The script uses these public compiler modes and target flags:

```sh
xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -emit-silgen -o oracle.sil Oracle.swift
xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target TARGET_TRIPLE -sdk SDK_PATH -Onone -emit-ir -o oracle.ll Oracle.swift
xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target TARGET_TRIPLE -sdk SDK_PATH -Onone -emit-assembly -o oracle.s Oracle.swift
xcrun swiftc -parse-as-library -module-name SwiftAsyncThunkProof -target TARGET_TRIPLE -sdk SDK_PATH -Onone -emit-object -o oracle.o Oracle.swift
xcrun --sdk SDK_NAME clang -std=c11 -target TARGET_TRIPLE -isysroot SDK_PATH -S -emit-llvm -O2 -o clang.ll Thunk.c
xcrun --sdk SDK_NAME clang -std=c11 -target TARGET_TRIPLE -isysroot SDK_PATH -S -O2 -o clang.s Thunk.c
xcrun --sdk SDK_NAME clang -std=c11 -target TARGET_TRIPLE -isysroot SDK_PATH -c -O2 -o clang.o Thunk.c
```

The host library inspection build adds `-emit-library` with the same `-parse-as-library`, module, target, SDK, and `-Onone` settings. The Swift and Clang commands exited successfully without diagnostics. Clang feature checks used public `__has_attribute(swiftasynccall)`, `__has_attribute(swift_async_context)`, and `__has_extension(swiftasynccc)`.

Focused validation:

```sh
sh interop/swift-abi-core/tests/check-swift-async-thunk.sh  # passed; Clang lowering matches compiler-derived entry signatures
sh -n interop/swift-abi-core/tests/check-swift-async-thunk.sh
cargo fmt --all -- --check
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
```

The script verifies SIL/LLVM signatures, exact compiler-emitted symbols, Swift caller task/resume/error evidence, Clang attribute availability, Clang IR signature and tail call, ARM64 branch/object references, host linkage imports, and host library dependencies. Device and simulator outputs are compile-only. No unit-test harness, production ABI API, async runtime state, target link, or invocation is part of this workstream.

## References

- [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html)
- [Swift ABI Calling Convention Summary](https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst)
- [Swift async/await proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md)
