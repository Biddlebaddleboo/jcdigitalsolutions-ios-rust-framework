# Swift Async Runtime Entry Feasibility

## Scope and result

**Result: no-go for a direct Rust/C or C++ caller under the inspected public interfaces.** The active Swift SDK exposes public task and continuation APIs to Swift source, but the installed C header/module-map surface and compiler-generated C++ header do not expose a supported task-entry function or a callable async function. The SDK export stubs list Swift concurrency task symbols, but export presence is not a public C/C++ contract.

This is a scoped audit, not a claim that no internal runtime entry exists. It does not call `swift_task_*`, invoke a Swift function, build a task context, or claim an async ABI/Rust `Future` bridge. The prior [C5 async thunk proof](ASYNC_THUNK_FEASIBILITY.md) shows that Clang can express a compiler-derived async function entry convention; that result does not supply the missing public task/context creation and resume contract.

## Toolchain, targets, and interfaces searched

Recorded on macOS `26.6.2` build `25G83`:

| Item | Value |
|---|---|
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| Xcode | `Xcode 26.6`, build `17F113` |
| Swift driver | `swift-driver version: 1.148.6` |
| Swift | `Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1.101)` |
| Clang | `Apple clang version 21.0.0 (clang-2100.1.1.101)` |
| Host target / macOS SDK | `x86_64-apple-macosx26.0`; SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk` |
| iOS device target / SDK | `arm64-apple-ios17.0`; SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk` |
| iOS simulator target / SDK | `arm64-apple-ios17.0-simulator`; SDK `26.5` at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk` |

Xcode 26.6 is the installed audit environment and is below the repository's Xcode `27.x` baseline. The installed device `_Concurrency.swiftinterface` identifies its interface compiler as Swift `6.3.2`; the active command-line driver reports Swift `6.3.3`.

The audit read or searched:

- Toolchain headers: `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include` and its `swift/` subtree. The installed `swift/` include subtree contains `bridging` and `bridging.modulemap`; it has no installed `swift/Runtime` or `swift/ABI` task-entry header.
- iPhoneOS and iPhoneSimulator public C headers/module maps under each SDK's `usr/include`, plus the compiler shim directory `usr/lib/swift/shims`.
- Device interface: `.../iPhoneOS26.5.sdk/usr/lib/swift/_Concurrency.swiftmodule/arm64e-apple-ios.swiftinterface`.
- Simulator interface: `.../iPhoneSimulator26.5.sdk/usr/lib/swift/_Concurrency.swiftmodule/arm64-apple-ios-simulator.swiftinterface`.
- The installed `_SwiftConcurrency.h` and `module.modulemap` shim, and the iPhoneOS/simulator `usr/lib/swift/libswift_Concurrency.tbd` export stubs.
- A Swift compiler-generated C++ header for temporary `async` functions and synchronous wrappers returning `Task`.
- Compiler-generated device and simulator LLVM IR, plus device SILGen, for temporary async callers and task creation.

The header/module-map search looked for `swift_task_create`, `swift_task_alloc`, `swift_task_dealloc`, `swift_task_switch`, `swift_task_enqueue`, and `swift_task_resume`. It found no declaration of those names in the searched C headers or module maps.

## Public Swift API is not a C/C++ task-entry contract

The installed `_Concurrency` interface declares public `Task<Success, Failure>` and Swift initializers accepting an escaping async closure. The iOS declaration has `@available(iOS 13.0, *)`; the exact nonthrowing initializer form is:

```swift
public init(name: Swift.String? = nil, priority: _Concurrency.TaskPriority? = nil, @_inheritActorContext @_implicitSelfCapture operation: sending @escaping @isolated(any) () async -> Success)
```

It is a Swift-language API, not an `extern "C"` declaration. The interface body delegates creation to the compiler builtin `Builtin.createTask`.

Apple documents `Task` as a unit of asynchronous work: a task's operation can start after task creation without a separate scheduling call; the `Task` value provides result access and cancellation. These are Swift API semantics and do not document how a C or C++ caller constructs the task's async closure, task context, executor state, or resume path. The [`Task` documentation](https://developer.apple.com/documentation/swift/task) is the supported Swift entry surface found in this audit.

The installed `_Concurrency.swiftinterface` also contains underscore-prefixed Swift declarations bound with `@_silgen_name`: public `_getCurrentAsyncTask() -> Builtin.NativeObject?` to `swift_task_getCurrent`, public `_taskFutureGet<T>(_ task: Builtin.NativeObject) async -> T` to `swift_task_future_wait`, public `_taskFutureGetThrowing<T>(_ task: Builtin.NativeObject) async throws -> T` to `swift_task_future_wait_throwing`, and public `_taskCancel(_ task: Builtin.NativeObject)` to `swift_task_cancel`. The current-task/future/cancel declarations take `Builtin.NativeObject`; the future wait functions themselves are async. The interface also marks `_enqueueJobGlobal(_ task: Builtin.Job)` to `swift_task_enqueueGlobal` as internal and uses `Builtin.Job`. These are observations of underscore-prefixed Swift-module implementation bindings, not documented C/C++ APIs or a task-creation/context contract. They operate on existing Swift runtime task objects; none defines a Rust-callable initializer for the task context. The documented Swift surface remains `Task` and its supported methods, not these internal-name bindings.

The public `withCheckedContinuation` and `withUnsafeContinuation` functions are also async Swift APIs. Apple describes a checked continuation as an opaque representation of program state for the **current task**; a continuation resume must occur exactly once on every path. It resumes an already-running task at its suspension point; it does not create a task-entry context for a C/Rust caller. See [CheckedContinuation](https://developer.apple.com/documentation/swift/checkedcontinuation) and [SE-0296: Async/await](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md).

SE-0296 states that async code runs in the context of a task and that async entry is a separate task/structured-concurrency concern. Swift's `@main` async program entry and `Task` construction are Swift compiler/language facilities, not a documented C/C++ task-entry runtime API. The public [Structured Concurrency proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md) and [Swift ABI Calling Convention Summary](https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst) describe Swift task and calling-convention concepts; neither supplies a C/Rust task creation/resume contract.

## Generated caller evidence

The temporary compiler-oracle source declared `asyncIdentity`, `asyncThrowsIdentity`, async callers for each, and synchronous `launchTask` / `launchThrowingTask` functions that return `Task<Int32, Never>` and `Task<Int32, any Error>`. The script compiled SILGen and LLVM IR with `-Onone` for iOS device and simulator; no function was invoked.

In this Swift 6.3.3 device IR, async caller continuations:

- obtain a context-frame size from compiler-emitted async-function metadata and call `swift_task_alloc`;
- store the parent async context and a compiler-generated `TQ0_` continuation function in the allocated context before tail-calling the async entry;
- resume through that generated continuation, call `swift_task_dealloc`, and forward the result;
- use `swift_task_switch` on the compiler-emitted resume path where the throwing continuation suspends;
- use `swiftself null` on the observed success continuation and a `swiftself` error pointer on the observed failure continuation;
- include `hop_to_executor` in SIL. The `Task` initializer lowering derives closure isolation and passes an initial-executor option to its task-creation builtin.

These are per-fixture compiler observations, not a general task-frame, executor, error-ownership, or cancellation ABI. In particular, the emitted code does not tell a C/Rust caller what values or lifecycle to supply for an arbitrary Apple async function.

The generated Task initializer call in device LLVM IR is:

```llvm
declare swiftcc %swift.async_task_and_context @swift_task_create(i64, ptr, ptr, ptr, ptr)
```

The same probe emits `swiftcc ptr @swift_task_alloc(i64)`, `swiftcc void @swift_task_dealloc(ptr)`, and `swifttailcc void @swift_task_switch(ptr, ptr, i64, i64)`. Those exact forms are output from this Xcode/target/optimization probe only; they are not proposed Rust declarations. The different Swift calling conventions, compiler closure/resume pointers, metadata-derived frame size, executor value, and result/error continuations are not described by a supported C contract.

## C and C++ surface results

The installed SDK's `_SwiftConcurrency.h` shim defines only `_SwiftContext` with a `parentContext` pointer. Its `_SwiftConcurrencyShims` module map does not declare task creation or resume operations. The shim is installed under `usr/lib/swift/shims`, not the public `usr/include/swift` runtime header tree.

The iPhoneOS and iPhoneSimulator `libswift_Concurrency.tbd` stubs list exported symbols including:

```text
_swift_task_create
_swift_task_create_common
_swift_task_alloc
_swift_task_dealloc
_swift_task_switch
_swift_task_enqueueGlobal
_swift_task_cancel
```

The iPhoneOS stub declares target `[arm64e-ios]`; the iOS Simulator stub declares `[x86_64-ios-simulator, arm64-ios-simulator]`; both report Swift ABI version `7`. They contain no C declarations or per-function task-context, executor, cancellation, error, ownership, or deployment contract. An exported name is not sufficient evidence of a supported public API; none of these names was called. The device compiler-oracle IR target was `arm64-apple-ios17.0`, while this SDK's observed device export stub lists `arm64e-ios`. Because this audit did not link a device object or archive, it makes no claim about whether or how those slices can be linked for a product target.

Public C++ header generation succeeded with zero stdout/stderr for the temporary fixture, but emitted only unavailable comments for the candidate wrappers:

```text
// Unavailable in C++: Swift global function 'asyncIdentity(_:)'.
// Unavailable in C++: Swift global function 'asyncThrowsIdentity(_:)'.
// Unavailable in C++: Swift global function 'launchTask(_:)'.
// Unavailable in C++: Swift global function 'launchThrowingTask(_:)'.
```

No callable C++ thunk was emitted for either async entry or either `Task`-returning wrapper. The exact omission is toolchain evidence for this fixture, not a claim about all future Swift C++ interop modes.

## `OperationState<T,E>` boundary

[`framework-async::OperationState<T,E>`](../../crates/framework-async/src/operation.rs) is a no-allocation Rust completion/waker cell. `new()` starts no task; `start()` only changes the local operation phase; the unique `complete()` winner publishes a `Completion<T,E>` and wakes the registered Rust `Waker`; the Rust future later consumes that value. Cancellation races select the Rust operation's terminal outcome.

It does not allocate a Swift async context, create or schedule a Swift task, select a Swift executor, install a Swift resume function, transfer a Swift error, or invoke Swift task teardown. It can be a Rust-side completion component only after a supported Swift task-entry and return path exists.

## Scoped no-go and evidence limits

No supported public C/C++ task-entry interface was found in the inspected Xcode 26.6/SDK 26.5 headers, module maps, interfaces, documentation, or generated C++ header surface. The precise blocker is that the only documented task-creation facility found is a Swift `Task` API whose async closure and task behavior are not callable through the generated C++ API; the compiler-emitted runtime calls needed to create/allocate/resume tasks have no corresponding public C/Rust contract in the inspected headers or documentation.

This audit does **not** claim that `swift_task_*` symbols do not exist or are absent from the linker exports. The SDK stubs prove that several such names are exported. It also does not prove that no future Swift release will add a supported C interface. It does not establish general Swift task-context layouts, executor rules, Swift error ownership, cancellation bridging, OS availability of the exported task symbols, or behavior beyond the fixture. The iOS device/simulator outputs are compiler IR/SIL only: no object link, runtime call, device execution, or archive validation was attempted.

All Swift input, generated headers, SIL, LLVM IR, and libraries were created under an external temporary directory. No `.swift` file or production wrapper was added to the checkout.

## Reproduction and validation

Run on macOS with Xcode:

```sh
sh interop/swift-abi-core/tests/check-swift-async-runtime-entry.sh
```

The script writes all compiler-oracle artifacts outside the checkout and removes them on exit. Set `KEEP_SWIFT_ASYNC_RUNTIME_PROBE=1` to preserve them outside the checkout. It checks the installed interfaces, C headers/module maps, SDK export stubs, compiler-generated device/simulator context and continuation patterns, and generated C++ header omissions. It never calls a runtime task symbol.

Validation for this handoff:

| Check | Result |
|---|---|
| `sh interop/swift-abi-core/tests/check-swift-async-runtime-entry.sh` | Pass on macOS/Xcode listed above |
| `sh -n interop/swift-abi-core/tests/check-swift-async-runtime-entry.sh` | Pass |
| `cargo fmt --all -- --check` | Pass |
| `cargo xtask docs-check` | Pass |
| `cargo xtask zero-swift-source` | Pass |
| `git diff --check` | Pass |

No audit probe command failed. Header/module-map search and C++ header generation returned no supported task-entry declaration; those negative findings are the audit result, not a tool failure. No device object/archive link, device execution, or runtime task-symbol call was attempted.

## References

- [Apple Task documentation](https://developer.apple.com/documentation/swift/task)
- [Apple CheckedContinuation documentation](https://developer.apple.com/documentation/swift/checkedcontinuation)
- [SE-0296: Async/await](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md)
- [SE-0304: Structured concurrency](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)
- [Swift ABI Calling Convention Summary](https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst)
- [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html)
