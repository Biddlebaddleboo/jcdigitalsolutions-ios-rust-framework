# D56 — RoomPlan device-support status

## Status

The bounded portable result and iOS query are complete in this isolated worktree. D56 is reserved
here; D55/B60 belongs to StoreKit. Root manifest, CI, and global capability status remain outside
this worktree and need parent integration.

## Scope

- Add `framework-roomplan`, a `no_std`, allocation-free value for the result of a RoomPlan device
  support query.
- Add `ios-roomplan::device_support()` for the public static
  `RoomCaptureSession.isSupported` property, introduced with RoomPlan on iOS 16.0.
- The backend must not create or run `RoomCaptureSession`, access image or LiDAR data, request
  permission, present UI, or claim that scanning will succeed.
- Keep all generated or handwritten `.swift` source out of the repository and shipping build.
- Do not edit the global capability manifest, root plans, CI, or unrelated capability crates.

## ABI feasibility

The installed iOS SDK does not declare `RoomCaptureSession` in Objective-C and there is no typed
`objc2` binding for its static property. The public Swift interface declares
`RoomCaptureSession.isSupported: Bool`. A temporary compiler oracle must derive the exact metadata
accessor and getter lowering for device and Simulator targets; Clang `swiftcall` must match both
symbols and the 64-bit metadata response. The getter argument uses Clang `swift_context`, which
lowers to `swiftself` in the oracle IR. See [Clang's `swiftcall` ABI attributes](https://clang.llvm.org/docs/AttributeReference.html#swiftcall)
for the context-parameter rule. The oracle source exists only in a temporary directory that the
package gate removes. The C thunk uses the public framework's exported symbols and is the smallest
Rust-callable path under the repository's zero-Swift-source rule.

## Acceptance

- The portable result compiles without `std` and has no unsafe code or platform types.
- iOS reports only Apple's `isSupported` result; non-iOS returns `None`.
- Rust compiles for iOS device and arm64 Simulator.
- Device and Simulator compiler-oracle checks match the metadata accessor and getter ABI, including
  the `swift_context`/`swiftself` parameter treatment.
- Release probes link against `RoomPlan.framework`, import the expected public symbols, declare
  iOS 16.0 minimum, and are never executed.
- Package docs state that support does not imply authorization, active scanning, or scan success.

## Evidence

On macOS with Xcode 26.6 build 17F113, Swift 6.3.3, Apple Clang 21.0.0, and iOS SDK 26.5:

- `sh platform/ios/ios-roomplan/check.sh` passed. This ran format, portable and host checks, strict
  Clippy, rustdoc, iOS device and arm64 Simulator checks, C/Swift ABI comparison, and Release
  link/import inspection.
- `sh platform/ios/ios-roomplan/check-swiftcall.sh` matched the Swift compiler oracle for the
  metadata accessor `_$s8RoomPlan0A14CaptureSessionCMa` and getter
  `_$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ` on arm64 device and arm64 Simulator,
  including the getter's `swift_context`/`swiftself` parameter.
- Device Release probe imports only `/System/Library/Frameworks/RoomPlan.framework/RoomPlan` and
  `/usr/lib/libSystem.B.dylib`; Simulator imports the same two paths. Both declare `minos 16.0`
  with SDK 26.5 and retain both public Swift symbols as imports.
- Neither Release probe was run. No physical-device result or live scan is claimed.
- The package has no `.swift` source. The one-line compiler oracle exists only in a temporary path
  that the gate removes on exit.
- The SDK Swift interface marks `RoomCaptureSession` available from iOS 16.0, and its `RoomPlan.tbd`
  exports both symbols above; the Objective-C umbrella headers declare neither the class nor the
  property.

The official [`RoomCaptureSession.isSupported` documentation](https://developer.apple.com/documentation/RoomPlan/RoomCaptureSession/isSupported)
defines the value as device support and says it is true when a LiDAR Scanner is present. That does
not promise a successful scan.

The repository baseline calls for Xcode 27.x. Local evidence uses Xcode 26.6, so the Xcode 27.x
gate remains for parent CI. No runtime or physical-device evidence was collected.
