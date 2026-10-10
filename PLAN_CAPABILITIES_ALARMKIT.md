# PLAN_CAPABILITIES_ALARMKIT.md — B280 authorization state

## Installed tooling scope for future work

R1/R2 shared tooling is completed. Read `docs/SHARED_TOOLING.md` rather than implementing or inspecting shared tooling source. The configured `ios-alarmkit-status` pilot is available through `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-alarmkit-status`; inspect `--explain ios-alarmkit-status` for exactly what it covers. The registered validator checks the scoped status pilot only; it does not prove alarm scheduling or delivery on a real device. Retain every historic D/B/F acceptance limit and compiler-oracle requirement; for new coverage add declarative schema-v1 gates and optional bounded adapters without weakening negative tests or treating unexecuted runtime checks as passed.

## Scope

B280 implements one read-only AlarmKit operation: the current app's `AlarmManager.authorizationState` snapshot on iOS 26.0+. It does not implement alarm configuration, scheduling, listing, cancellation, pause/resume/stop, authorization requests, or update streams

## Contract

`ios-alarmkit-status::authorization_state()` returns `NotDetermined`, `Denied`, `Authorized`, or `Unknown`. `Unknown` preserves forward compatibility if Apple adds a public case. The value is point-in-time state only; it does not guarantee that a later alarm operation will succeed

The API does not call `requestAuthorization()`, prompt the person, schedule or alter an alarm, inspect alarm content, or subscribe to updates. It uses only `AlarmManager.shared` and the synchronous `authorizationState` property

## Compiler and ABI evidence

The installed Xcode 26.6 build `17F113` / iOS SDK 26.5 public `AlarmKit.swiftinterface` declares iOS 26.0 `AlarmManager`, the public `AuthorizationState` cases `notDetermined`, `denied`, and `authorized`, and the synchronous `authorizationState` getter. The device interface lists the authorization-state enum as a public resilient enum without a frozen layout or integer raw type. `AlarmKit.tbd` exports the manager metadata accessor, `shared` getter, property getter, enum metadata accessor, and one compiler-emitted case-tag symbol for each public case

The temporary Swift oracle input was `import AlarmKit; public func alarmAuthorizationIsAuthorizedOracle() -> Bool { switch AlarmManager.shared.authorizationState { case .authorized: return true; default: return false } }`. It was compiled for `arm64-apple-ios26.0` with `xcrun swiftc -parse-as-library -emit-ir -O -module-name AlarmKitOracle -target arm64-apple-ios26.0 -sdk <iphoneos-sdk-path> -o <temporary-dir>/oracle.ll <temporary-dir>/Oracle.swift`. The property getter lowered as `swiftcc void(ptr noalias sret(%swift.opaque), ptr swiftself)`. A `switch` over the enum requested metadata, loaded the value-witness table at metadata offset `-1`, used `getEnumTag`, compared its result with the exported `authorized` case tag, and called the `destroy` witness. The Clang C thunk's emitted LLVM IR matches that path: metadata response `{ ptr, i64 }`, the manager `swiftself` getter, an indirect enum result, dynamic VWT size/alignment, `getEnumTag`, and `destroy`

The implementation does not assume an enum byte layout, hard-code tag numbers, copy opaque enum bytes, or construct Swift values. It allocates using metadata size and alignment, reads the tag using the enum value witness, destroys the initialized value using its witness, then maps compiler-emitted case tags to Rust variants. The manager's owned Swift class reference moves to `SwiftRetained` and releases through the existing `swift-abi-core/apple-runtime` path

Primary ABI references: Swift [type metadata](https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst), [value witness definitions](https://github.com/swiftlang/swift/blob/main/include/swift/ABI/ValueWitness.def), and Clang's [`swiftcall` and `swift_indirect_result` attributes](https://clang.llvm.org/docs/AttributeReference.html#swiftcall). Apple API references: [AlarmManager](https://developer.apple.com/documentation/alarmkit/alarmmanager), [AuthorizationState](https://developer.apple.com/documentation/alarmkit/alarmmanager/authorizationstate-swift.enum), and [`authorizationState`](https://developer.apple.com/documentation/alarmkit/alarmmanager/authorizationstate-swift.property)

## Platform and limits

AlarmKit is iOS 26.0+ and unavailable on Mac Catalyst in the inspected SDK. The crate's Rust surface targets iOS device and arm64 Simulator. The static C bridge strongly links `AlarmKit`; hosts must build with iOS 26.0 or later. The Xcode 26.6 / SDK 26.5 host is below this repository's Xcode 27.x baseline

No device/Simulator runtime query, authorization request, alarm operation, app archive/link, or Apple behavior-parity check was performed. Compile and static ABI evidence does not establish runtime behavior. Recheck the public interface, exported symbols, and compiler lowering on the Xcode 27.x baseline

## Static gates

`platform/ios/ios-alarmkit-status/check.sh` runs formatting, host/device/Simulator `cargo check`, strict Clippy, rustdoc, `docs-check`, zero-Swift-source, and scoped whitespace checks. No Rust unit or integration tests are added by B280
