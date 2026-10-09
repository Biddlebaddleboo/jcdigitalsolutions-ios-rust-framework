# SwiftRetained Runtime Ownership Proof

## Scope

This proof covers only the existing opt-in `SwiftRetained` strong-reference wrapper for native Swift class pointers. It checks `SwiftRetained::from_owned_ptr`, `SwiftRetained::retain_borrowed`, `Clone`, `Drop`, `into_raw`, and re-adoption via `from_owned_ptr`. It does not add runtime behavior or claim Swift method-call, metadata, resilient value, `String`, `Optional`, throws, async, Translation, StoreKit, or App Intents support.

## Repro command and toolchain

Run on macOS with Xcode and the Rust host target:

```sh
sh interop/swift-abi-core/tests/check-retained-ownership.sh
```

The proof creates its Swift class fixture, Rust caller, objects, and temporary Cargo package in a `mktemp` directory outside the checkout. An exit trap removes that directory. The fixture uses a synchronous `deinit` counter; it has no sleep, external service, or committed/generated shipping Swift source.

Recorded local toolchain:

- Xcode 26.6, build 17F113.
- Swift 6.3.3, `swift-driver` 1.148.6 (`swiftlang-6.3.3.1.3 clang-2100.1.1.101`).
- macOS 26.6.2; host target `x86_64-apple-darwin`.
- Rust 1.94.1, LLVM 21.1.8.
- iPhoneOS and iPhoneSimulator SDK 26.5.
- Swift oracle targets `arm64-apple-ios15.0` and `arm64-apple-ios15.0-simulator`.

The script reads the active Swift host runtime paths from `xcrun swiftc -print-target-info`. The recorded `runtimeLibraryPaths` are `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx` and `/usr/lib/swift`; the recorded `librariesRequireRPath` value is `false`. Rust's host link still emits an `@rpath/libswiftCore.dylib` dependency, so the script uses the system runtime path `/usr/lib/swift` as its host `LC_RPATH`. It does not add the toolchain build-runtime directory as a second runtime path. The Rust host link reports:

```text
_swift_release
_swift_retain
@rpath/libswiftCore.dylib (compatibility version 0.0.0, current version 0.0.0)
LC_RPATH /usr/lib/swift
dyld load: /usr/lib/swift/libswiftCore.dylib
```

`DYLD_PRINT_LIBRARIES=1` confirms exactly one loaded `libswiftCore.dylib`. The first probe attempt used the toolchain's `usr/lib/swift-5.0/macosx` runtime path and loaded a duplicate Swift runtime beside macOS's system runtime; it emitted duplicate-class warnings and aborted. The committed gate therefore selects the system runtime path reported by the active compiler.

## Compiler-derived ABI evidence

The ephemeral Swift source invokes `Unmanaged<OwnershipProbe>.retain()` and `.release()`. Swift 6.3.3 emits these host, device, and simulator LLVM IR declarations and calls:

```llvm
declare ptr @swift_retain(ptr returned)
declare void @swift_release(ptr)
```

The declarations are the C ABI emitted by the compiler, not signatures inferred from a guessed Swift calling convention. Host runtime execution also confirms that the linked Rust wrapper reaches these imports.

## Runtime result

For each of 64 fresh Swift class instances, the Rust host caller:

1. Adopts the one strong reference returned by Swift with `from_owned_ptr`.
2. Adds a strong reference with `retain_borrowed` and another with `Clone`.
3. Drops the first wrapper and confirms `deinit` has not run.
4. Transfers the cloned reference with `into_raw`, re-adopts it with `from_owned_ptr`, drops the borrowed wrapper, and confirms `deinit` has not run.
5. Drops the final wrapper and confirms the `deinit` count increases exactly once.

The host Rust binary links and runs on macOS. The Swift compiler oracle emits device and simulator LLVM IR and object files. The proof script also runs these Rust checks for both iOS targets:

```sh
cargo check --locked --offline -p swift-abi-core --features apple-runtime --target aarch64-apple-ios
cargo check --locked --offline -p swift-abi-core --features apple-runtime --target aarch64-apple-ios-sim
cargo check --locked --offline -p swift-abi-generated --features apple-runtime --target aarch64-apple-ios
cargo check --locked --offline -p swift-abi-generated --features apple-runtime --target aarch64-apple-ios-sim
```

Those target results are compile-only; no Rust iOS app archive/link, simulator execution, or device execution is claimed.

The default `swift-abi-core` feature path also builds a host executable; `otool -L` has no `libswiftCore.dylib` dependency and `nm -u` has no `_swift_retain` or `_swift_release` import.

Additional focused validation passed:

```sh
cargo fmt --all -- --check
cargo clippy --locked --offline -p swift-abi-core -p swift-abi-generated --all-targets --all-features -- -D warnings
cargo test --locked --offline -p swift-abi-core -p swift-abi-generated
RUSTFLAGS='-C link-arg=-Wl,-rpath,/usr/lib/swift' cargo test --locked --offline -p swift-abi-core -p swift-abi-generated --all-features
git diff --check
```

Both crate test commands pass with zero unit or doc tests defined. The all-features test harness needs the host `LC_RPATH` because `apple-runtime` links `swiftCore`; the live ownership assertions run in the proof script above.

## Limits

This is class-reference lifetime evidence only. It does not prove concurrent retain/drop safety, `Send`/`Sync`, Swift method invocation, value layout or witnesses, any Swift value adapter, Apple API behavior, or device/simulator runtime availability. `SwiftRetained` remains opt-in and keeps the existing thread-affinity marker.
