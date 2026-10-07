# Generated Swift Runtime Ownership Fixture

Generated from ephemeral compiler-oracle input on 2026-10-07. The temporary `.swift` input was created outside the repository and deleted when the probe exited. No Swift source is shipped or required by either crate.

## Compiler invocation

For each target, a temporary native Swift class reference was returned from a function and removed from an optional global. The installed Swift compiler emitted LLVM IR with these runtime references:

```text
xcrun swiftc -sdk <iphoneos-sdk> -target arm64-apple-ios15.0 \
  -module-name SwiftAbiOwnershipOracle -parse-as-library -emit-ir -Onone \
  <temporary-input.swift> -o <temporary-output.ll>

xcrun swiftc -sdk <iphonesimulator-sdk> -target arm64-apple-ios15.0-simulator \
  -module-name SwiftAbiOwnershipOracle -parse-as-library -emit-ir -Onone \
  <temporary-input.swift> -o <temporary-output.ll>
```

## Exact emitted declarations and calls

Both device and simulator LLVM IR emitted the same lines:

```llvm
call void @swift_release(ptr %2)
declare void @swift_release(ptr)
%1 = call ptr @swift_retain(ptr returned %0)
declare ptr @swift_retain(ptr returned)
```

No `swiftcc` or `swifttailcc` calling convention is attached to these runtime calls; the declarations use LLVM's default C calling convention. This is the evidence for the generated crate's two FFI signatures. It is not evidence for a Swift method-call thunk.

The iOS 26.5 device SDK exports Mach-O symbols `_swift_retain` and `_swift_release` from `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/usr/lib/swift/libswiftCore.tbd`. The simulator SDK exports the same symbols from `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk/usr/lib/swift/libswiftCore.tbd`. Rust source-level `#[link_name]` values omit the Mach-O leading underscore: `swift_retain` and `swift_release`.

A temporary Clang C dylib link probe for each SDK used target `arm64-apple-ios15.0` or `arm64-apple-ios15.0-simulator` with `-isysroot <SDK> -dynamiclib -lswiftCore`. Both links exited 0. `nm -g` showed undefined imports `_swift_retain` and `_swift_release`; `otool -L` showed `/usr/lib/swift/libswiftCore.dylib`. This confirms the SDK link stub/path for Clang, not a Rust application archive or runtime execution.

## Scope

The fixture supports only raw Swift class-reference retain/release with compiler-derived C ABI signatures. It does not establish metadata/value-witness layouts, Swift String construction, Swift function calling conventions, throws, async continuations, or Translation/StoreKit/App Intents support.
