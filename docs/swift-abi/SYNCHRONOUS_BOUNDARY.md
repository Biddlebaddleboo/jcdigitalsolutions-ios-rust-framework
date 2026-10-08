# Synchronous Swift ABI Boundary

The scalar-only Clang thunk proof is recorded in [Synchronous Swift Thunk Feasibility](SYNCHRONOUS_THUNK_FEASIBILITY.md). It does not change the unsupported status for direct Apple framework method calls below

## Implemented

`swift-abi-core` is a separate optional crate. With its `apple-runtime` feature disabled (the default), it links no Swift runtime. With the feature enabled for an Apple target, it provides an owned strong-reference wrapper for Swift class pointers. Clone uses one `swift_retain`; Drop uses one `swift_release`; raw ownership transfer is explicit. It adds no Rust `Arc`/`Rc` layer and does not mark Swift objects `Send` or `Sync`. The live macOS host class-lifetime proof is recorded in [`SwiftRetained Runtime Ownership Proof`](OWNERSHIP_RUNTIME_PROOF.md).

The raw `C` ABI declarations are backed by Swift 6.3.3 LLVM IR generated from temporary compiler-oracle input for host, device, and simulator targets. Exact iOS target declarations are preserved in [`runtime-ownership.md`](../../interop/swift-abi-generated/fixtures/runtime-ownership.md); the new fixture checks the same declarations for each target. The Rust host proof links and runs against one `@rpath/libswiftCore.dylib` loaded from `/usr/lib/swift`, and verifies 64 exact class destruction cycles across `SwiftRetained` ownership operations. The default feature path has no `swiftCore` linkage or runtime imports. iOS device and simulator Rust checks and Swift object builds are compile-only; no iOS application link or device/simulator runtime execution was run.

## Translation declaration and compiler lowering

The iOS 26.5 device interface is:

```text
@available(iOS 18.0, macOS 15.0, macCatalyst 26.0, *)
public class TranslationSession
public func translate(_ string: Swift.String) async throws -> Translation.TranslationSession.Response
@available(iOS 26.0, macOS 26.0, *)
convenience public init(installedSource source: Foundation.Locale.Language, target: Foundation.Locale.Language?)
```

The arm64 simulator interface exposes the same declarations. Notably, the installed-language initializer needed for the documented non-UI path is iOS 26.0+, while the session and single-string method are iOS 18.0+ in this SDK. A lower deployment target cannot assume the installed-language initializer.

An ephemeral Swift caller was compiled using Xcode 26.6 / Swift 6.3.3, `-emit-silgen`, `-emit-ir`, and `-O -emit-assembly` targets `arm64-apple-ios26.0` and `arm64-apple-ios26.0-simulator`. The initialized-languages path compiled on both targets. Its caller IR includes these compiler-derived constructor and metadata references:

```llvm
call swiftcc %swift.metadata_response @"$s11Translation0A7SessionCMa"(i64 0)
call swiftcc ptr @"$s11Translation0A7SessionC15installedSource6targetAC10Foundation6LocaleV8LanguageV_AJSgtcfC"(ptr noalias %.reload43, ptr noalias %.reload25, ptr swiftself %9)
```

The device SDK's public `Translation.tbd` exports both exact symbols, as well as the async method descriptor below. The init's SIL signature takes `@in Locale.Language`, `@in Optional<Locale.Language>`, and a `@thick TranslationSession.Type`, returning an owned `TranslationSession`. The lowering therefore requires construction of resilient Foundation values before the owned class reference can exist.

The SIL type for the translation method call is:

```text
$@convention(method) @async (@guaranteed String, @guaranteed TranslationSession) ->
  (@out TranslationSession.Response, @error any Error)
```

The caller's LLVM IR includes this async call shape (the `translateOne` caller is an oracle wrapper, not repository source):

```llvm
define swifttailcc void @"$s25SwiftAbiTranslationOracle12translateOney0C00C7SessionC8ResponseVAE_SStYaKF"(ptr noalias %0, ptr swiftasync %1, ptr %2, i64 %3, ptr %4)
...
%8 = add i64 ptrtoint (ptr @"$s11Translation0A7SessionC9translateyAC8ResponseVSSYaKFTjTu" to i64), %7
%9 = inttoptr i64 %8 to ptr
%12 = call swiftcc ptr @swift_task_alloc(i64 %11)
musttail call swifttailcc void %9(ptr noalias %0, ptr swiftasync %12, i64 %3, ptr %4, ptr swiftself %2)
...
define internal swifttailcc void @"$s25SwiftAbiTranslationOracle12translateOney0C00C7SessionC8ResponseVAE_SStYaKFTQ0_"(ptr swiftasync %0, ptr swiftself %1)
```

The optimized arm64 assembly for both targets includes:

```asm
adrp x8, _$s11Translation0A7SessionC9translateyAC8ResponseVSSYaKFTjTu@GOTPAGE
ldr x8, [x8, _$s11Translation0A7SessionC9translateyAC8ResponseVSSYaKFTjTu@GOTPAGEOFF]
bl _swift_task_alloc
```

It then tail-calls the indirect entry with the continuation and Swift context registers. The actual compiler-generated method descriptor is `$s11Translation0A7SessionC9translateyAC8ResponseVSSYaKFTjTu`. The device SDK `Translation.tbd` exports this exact symbol. Device and simulator lowered to the same `swifttailcc` call shape. This is not a plain synchronous `swiftcall`: it uses an async function descriptor, a Swift async context, a tail-called continuation entry, the String's two arm64 words, an indirect `Response` result buffer, and an error result carried through the async continuation.

## Explicitly unsupported

No Translation initializer or translation request has been invoked from Rust. No Clang/Rust thunk, Swift async-context allocation/lifetime adapter, result/error continuation, cancellation mapping, `Locale.Language`/`Optional` value witnesses, or `TranslationSession.Response` value-witness path exists. The compiler probe confirms the Swift-side lowering but does not produce a Rust-callable implementation. Because the remaining call path exceeds this synchronous ownership crate, end-to-end Translation is unsupported by this change; no Translation, StoreKit, or App Intents coverage is claimed.

## Linker validation gap

The crate passed Rust `cargo check` for device and simulator, but this does not link an application. A separate temporary `rustc --crate-type=cdylib -C panic=abort` FFI probe for each target reached Xcode `ld` and failed with `Assertion failed: (_sideInfo), function sideInfo, file Atom.h, line 284.` This cdylib probe is not the planned `rlib`-into-app archive shape and does not show that the crate fails to link; it leaves Rust app/archive linkage unverified. The separate Clang dylib probes recorded in the generated fixture passed.

Also unsupported by this change:

- direct synchronous calls to public Swift methods;
- `SwiftMetadata`, value-witness access, and resilient opaque value storage;
- Swift `String`, `Array`, `Optional`, or enum adapters;
- synchronous throwing functions;
- Swift async/throws, callbacks, cancellation, or `AsyncSequence`;
- generated Rust-defined Swift types, protocol conformances, or App Intents metadata.
