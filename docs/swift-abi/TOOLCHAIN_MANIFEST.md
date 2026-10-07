# Swift ABI Toolchain Manifest

Captured on 2026-10-07 on the Workstream C host before ABI implementation.

## Host and selected toolchain

| Command | Output |
|---|---|
| `uname -a` | `Darwin Johns-Mac-Pro.local 25.6.0 Darwin Kernel Version 25.6.0: Fri Jul 31 19:11:49 PDT 2026; root:xnu-12377.161.14~5/RELEASE_X86_64 x86_64` |
| `xcode-select -p` | `/Applications/Xcode.app/Contents/Developer` |
| `xcodebuild -version` | `Xcode 26.6` / `Build version 17F113` |
| `xcrun swiftc --version` | `swift-driver version: 1.148.6 Apple Swift version 6.3.3 (swiftlang-6.3.3.1.3 clang-2100.1.1)` / `Target: x86_64-apple-macosx26.0` |
| `rustc --version` | `rustc 1.94.1 (e408947bf 2026-03-25)` |
| `cargo --version` | `cargo 1.94.1 (29ea6fb6a 2026-03-24)` |
| `rustup target list --installed` | `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-darwin` |

The repository does not yet declare an iOS deployment target. The installed iOS SDK reports `DefaultDeploymentTarget = 26.5`, `MinimumDeploymentTarget = 12.0`, `SwiftConcurrencyMinimumDeploymentTarget = 15.0`, `SwiftOSRuntimeMinimumDeploymentTarget = 12.2`, and `SwiftSpanMinimumDeploymentTarget = 26.0`. Compiler-oracle target versions must be stated per fixture rather than inferred from the SDK version.

## Installed Apple SDKs

| SDK | `--show-sdk-path` | `--show-sdk-version` |
|---|---|---|
| `iphoneos` | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk` | `26.5` |
| `iphonesimulator` | `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator26.5.sdk` | `26.5` |

SDK metadata is at each SDK path's `SDKSettings.plist`. The platform minimum is iOS 12.0; this is not a framework-selected deployment target.

## Target triples

Rust target names installed in pinned Rust 1.94.1 are `aarch64-apple-ios` (device) and `aarch64-apple-ios-sim` (simulator); their target library directories resolve under `/Users/john/.rustup/toolchains/1.94.1-x86_64-apple-darwin/lib/rustlib/`. Swift/LLVM's compiler triple spellings differ; use `swiftc -print-target-info` with the concrete target and deployment version in each fixture.

The generic compiler probes were:

```text
xcrun swiftc -print-target-info -target aarch64-apple-ios
  target.triple       = aarch64-apple-ios
  target.moduleTriple = arm64-apple-ios
  target.platform     = iphoneos
  target.arch         = aarch64
  pointer width       = 64 bits

xcrun swiftc -print-target-info -target aarch64-apple-ios-sim
  target.triple       = aarch64-apple-ios-sim
  target.moduleTriple = arm64-apple-ios-sim
  target.platform     = iphoneos
```

The Rust simulator spelling `aarch64-apple-ios-sim` is not Swift's simulator triple and is treated by `swiftc` as `iphoneos`. Swift's actual generic simulator triple is:

```text
xcrun swiftc -print-target-info -target aarch64-apple-ios-simulator
  target.triple       = aarch64-apple-ios-simulator
  target.moduleTriple = arm64-apple-ios-simulator
  target.platform     = iphonesimulator
  target.arch         = aarch64
  pointer width       = 64 bits
```

For explicit deployment-target lowering probes, Xcode 26.6 accepts:

```text
xcrun swiftc -print-target-info -target arm64-apple-ios15.0
  target.triple       = arm64-apple-ios15.0
  target.moduleTriple = arm64-apple-ios
  target.platform     = iphoneos
  swift runtime compatibility = 5.5

xcrun swiftc -print-target-info -target arm64-apple-ios15.0-simulator
  target.triple       = arm64-apple-ios15.0-simulator
  target.moduleTriple = arm64-apple-ios-simulator
  target.platform     = iphonesimulator
  swift runtime compatibility = 5.5
```

The `15.0` deployment version above is an explicit ABI probe setting, not a declared V1 support floor. Translation's availability is later and must be checked independently.

## SDK interface locations

The selected public interface files include:

```text
/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/StoreKit.framework/Modules/StoreKit.swiftmodule/arm64e-apple-ios.swiftinterface
/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/Translation.framework/Modules/Translation.swiftmodule/arm64e-apple-ios.swiftinterface
/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/Foundation.framework/Modules/Foundation.swiftmodule/arm64e-apple-ios.swiftinterface
```

These are SDK public module interfaces, not a substitute for per-symbol availability or ABI-lowering evidence. No private interface is in scope.

## Reproduction commands

```sh
xcode-select -p
xcodebuild -version
xcrun swiftc --version
xcrun --sdk iphoneos --show-sdk-path
xcrun --sdk iphoneos --show-sdk-version
xcrun --sdk iphonesimulator --show-sdk-path
xcrun --sdk iphonesimulator --show-sdk-version
xcrun swiftc -print-target-info -target aarch64-apple-ios
xcrun swiftc -print-target-info -target aarch64-apple-ios-sim
xcrun swiftc -print-target-info -target arm64-apple-ios15.0
xcrun swiftc -print-target-info -target arm64-apple-ios15.0-simulator
xcrun swiftc -print-target-info -target aarch64-apple-ios-simulator
```

The version/paths above were captured directly from these commands on this host. Compiler oracle inputs remain temporary and are not repository or shipping Swift source.
