# PLAN_CAPABILITIES_APP_INTENTS.md — D99: App Intents Build Metadata for Row 110

## Objective

Re-audit canonical row 110 against local Xcode 26.6 / iOS SDK 26.5, C7's Stage 0 evidence, and current public Apple documentation. Decide whether a documented Rust/C metadata input or public metadata-processor API has changed the row's status

## Status and recommendation

Keep row 110 `X` / `unsupported`. The current local toolchain remains below the plan's Xcode 27.x baseline. C7's Stage 0 result remains intact: a normal Swift build produced `Metadata.appintents`, but the processor executable, arguments, generated file lists, and metadata representation were not documented Rust/C inputs. Current Apple documentation describes Swift `AppIntent` and `AppIntentsPackage` protocols and compile-time metadata extraction; it does not document a Rust/C metadata schema, metadata-input format, or public processor invocation

The processor executable is still present at `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/appintentsmetadataprocessor`, and `xcrun --find appintentsmetadataprocessor` resolves it. Tool presence is not a public interface. This audit did not invoke the processor or inspect its private help/output

Stage 1 remains no-go for this audited toolchain. A Rust-defined `AppIntent` would need public type/protocol conformance and metadata contracts for compiler/system discovery, plus a supported route for the system to invoke `perform() async throws`. C6 still finds no documented public C/C++ task-entry/context/resume contract. Do not infer support from exported symbols, processor presence, compiler intermediates, or the public `xcodebuild` command alone

## Current local toolchain

Read-only host inspection on 2026-10-09:

| Item | Value |
| --- | --- |
| `sw_vers` | macOS 26.6.2, build 25G83 |
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| `xcodebuild -version` | Xcode 26.6, build 17F113 |
| `xcrun swiftc --version` | Swift 6.3.3, swift-driver 1.148.6; target `x86_64-apple-macosx26.0` |
| iPhoneOS SDK | 26.5, `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk` |
| iPhoneSimulator SDK | 26.5, `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk` |
| Plan baseline | Not met: installed Xcode 26.6 is below Xcode 27.x |

The metadata pipeline is a build-host capability. The Stage 0 simulator deployment target `17.0` is only the probe target, not a minimum deployment target for metadata extraction

## C7 Stage 0 evidence retained

C7's report is `docs/swift-abi/APP_INTENTS_STAGE0.md`; its temporary project and products were under `/private/tmp/jc-c7-appintents-stage0.o7vkLp`, outside the checkout. It built one parameterless Swift `AppIntent` whose `perform()` returned `.result()` with a normal Release generic iOS Simulator `xcodebuild` invocation. The build succeeded for simulator `arm64` and `x86_64` with SDK 26.5 and deployment target 17.0. No fixture, Swift source, metadata artifact, or build product was added to the repo

The verbose build log showed:

1. `SwiftEmitModule` and `SwiftCompile` for the temporary Swift declaration
2. `ExtractAppIntentsMetadata`, which invoked `appintentsmetadataprocessor` with compiler-derived module inputs, SDK/build/platform data, deployment target, bundle ID, target triples, linked binary, architecture dependency files, strings data, Swift source-file lists, dependency/static metadata lists, and Swift constant-value lists. Xcode wrote `Metadata.appintents` at the app bundle root
3. `AppIntentsSSUTraining`, which invoked `appintentsnltrainingprocessor` with the app plist, bundle/build paths, extracted metadata, dependency metadata list, and SSU archive option. The log said “No AppShortcuts found - Skipping”

C7 observed these steps only through Xcode's verbose build log. It did not call either processor directly, inspect or decode `Metadata.appintents`, construct metadata, or claim that observed arguments or formats are stable. The full command and build transcript remain in the C7 report

## Public API and metadata boundary

| Surface | Audited public contract | iOS floor |
| --- | --- | ---: |
| `AppIntent` | Swift protocol with metadata requirements and `perform() async throws`; `perform` returns an `IntentResult` | 16.0 |
| `AppIntentsPackage` | Swift protocol whose `includedPackages` property describes related package types | 17.0 |
| Metadata extraction | Apple says the compiler places App Intents metadata in each bundle at compile time; docs describe Swift declarations and package relationships | Build-time; no processor API floor documented |
| `INTENTS_CODEGEN_LANGUAGE` | Xcode build setting selects generated Intent class source language, `Swift` or `Objective-C`; it is not a Rust/C metadata input | Build setting; no runtime API floor |

Installed SDK evidence:

- `AppIntents.framework/Headers/AppIntents.h` imports Foundation and declares no C/Objective-C App Intent type, metadata input, or processor function
- `AppIntents.framework/Modules/AppIntents.swiftmodule/arm64e-apple-ios.swiftinterface` exposes `AppIntent` and `AppIntentsPackage` as Swift protocols; `AppIntent.perform()` is `async throws`
- The `AppIntent` protocol is available from iOS 16.0; `AppIntentsPackage` is available from iOS 17.0. These runtime API floors do not establish a build processor floor or a Rust metadata route
- The SDK and Xcode install contain `appintentsmetadataprocessor`, but Apple does not document its executable path, command line, input list formats, metadata schema, or output contract as a public Rust/C build interface

Apple's public docs state that the compiler extracts and places metadata at compile time, and that `AppIntentsPackage` is a Swift type relationship for intent declarations in shared frameworks. The public Xcode build setting `INTENTS_CODEGEN_LANGUAGE` only selects the language for generated Intent classes; it does not define a Rust/C declaration format or metadata processor API

## C6 runtime boundary

C7's metadata question and C6's invocation question are separate. C7 found no documented stable metadata input or public processor API. C6 found no supported public C/C++ task-entry/context/resume contract for invoking Swift async work. Because `AppIntent.perform()` is `async throws`, a metadata-only workaround would still not provide a supported Rust runtime entry. Do not fabricate Swift protocol witnesses, call private `swift_task_*` symbols, parse or emit private metadata, or invoke the observed processor command directly

## Acceptance boundary

This audit supports only:

- row 110 remains `X` on Xcode 26.6 / iOS SDK 26.5
- the local App Intents runtime protocol floors are iOS 16.0 for `AppIntent` and iOS 17.0 for `AppIntentsPackage`
- the normal Swift build can produce bundle metadata, as observed by C7; its processor inputs and metadata format remain undocumented implementation details
- re-audit is needed on a host that meets the Xcode 27.x baseline and only if Apple documents a stable Rust/C metadata input or processor interface, plus a supported async runtime entry for `perform()`

This audit does not establish any Rust App Intent, metadata artifact generator, runtime registration path, metadata processor invocation contract, or successful system-discovered intent

## Apple primary sources

- [AppIntent](https://developer.apple.com/documentation/appintents/appintent)
- [AppIntent.perform()](https://developer.apple.com/documentation/appintents/appintent/perform%28%29)
- [AppIntentsPackage](https://developer.apple.com/documentation/appintents/appintentspackage)
- [Configuring the runtime behavior of your app intents](https://developer.apple.com/documentation/appintents/configuring-the-runtime-behavior-of-your-app-intents)
- [Xcode Build settings reference](https://developer.apple.com/documentation/xcode/build-settings-reference)

## Root integration need

No Cargo, lock, CI, source, matrix, aggregate-plan, or docs-index change is needed. Row 110's current status reason and metadata-verification note remain accurate; keep the row `X` and retain the C7/C6 references
