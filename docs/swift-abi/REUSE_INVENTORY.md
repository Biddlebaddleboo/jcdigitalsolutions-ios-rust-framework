# Swift ABI Reuse Inventory

## Baseline and scope

This inventory covers `swift-abi-core`, `swift-abi-generated`, and the ActivityKit, AlarmKit, and Photogrammetry pilot bridges. The inspected toolchain is Xcode 26.6 / iOS SDK 26.5. It does not satisfy the planned Xcode 27.x qualification gate.

## Verified primitives

| Primitive | ABI evidence | Target and floor | Reuse and limits |
|---|---|---|---|
| Swift class retain/release | Swift 6.3.3 IR fixture in `interop/swift-abi-generated/fixtures/runtime-ownership.md`; host ownership probe in `interop/swift-abi-core/tests/check-retained-ownership.sh` | Host, arm64 iOS 15.0 device, arm64 iOS 15.0 Simulator | `SwiftRetained` is the shared owned-class wrapper. It covers `swift_retain` / `swift_release`, null rejection, clone, drop, and raw transfer. It does not assert `Send` or `Sync` |
| Metadata response | Per-pilot Swift IR and Clang IR oracles in ActivityKit and Photogrammetry; AlarmKit oracle record in `PLAN_CAPABILITIES_ALARMKIT.md` | ActivityKit iOS 16.1 device / arm64 Simulator; Photogrammetry iOS 17.0 device / arm64 Simulator; AlarmKit iOS 26.0 device oracle | `interop/swift-abi-core/include/swift_abi_runtime.h` shares only the 16-byte `{ pointer, uintptr_t }` layout and state offset 8. Framework symbols and calls stay local |
| Value-witness table layout | Photogrammetry `check-swiftcall.sh`, AlarmKit compiler-oracle record, and per-target C static assertions | Photogrammetry iOS 17.0 device / arm64 Simulator; AlarmKit iOS 26.0 device oracle; native x86_64 Simulator build has no matching Swift IR oracle | The shared header records destroy offset 8, size offset 64, flags offset 80, enum-tag witness offset 88, and table size 112. `swift_abi_value_storage_layout_from_metadata` shares the proven metadata-adjacent witness lookup and common nonzero-size/destroy/alignment requirements; existing helpers allocate storage and destroy/free. AlarmKit retains its enum-flag/tag checks, each bridge retains its own init, tag map, and error codes |
| Synchronous ActivityKit calls | `platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh` compares Swift and Clang IR for metadata, init, and `areActivitiesEnabled`; link audit checks weak imports | arm64 iOS 16.1 device and Simulator | Mangled symbols and call signatures stay in the ActivityKit bridge |
| Photogrammetry value calls | `platform/ios/ios-photogrammetry-status/check-swiftcall.sh` compares Swift and Clang IR for metadata, `isSupported`, `Limits`, and both `Int` getters | arm64 iOS 17.0 device and Simulator | `Limits` remains capability-local; the probe builds and inspects links but does not run |
| AlarmKit resilient enum call | Temporary Swift oracle, exported-symbol inspection, and matching Clang IR are recorded in `PLAN_CAPABILITIES_ALARMKIT.md` | arm64 iOS 26.0 device oracle; static Rust checks also cover arm64 Simulator | `authorizationState`, weak symbol checks, case tags, unknown-case mapping, and manager ownership stay in AlarmKit |

## Extraction boundary

The shared C header has ABI layout declarations, static layout assertions, metadata-to-witness validation, aligned alloc helpers, and destroy/free. `swift_abi_value_storage_layout_from_metadata` is C `static inline`; it uses no Rust `std`, heap alloc, global state, or dynamic dispatch. Only the oracle-proven AlarmKit enum and Photogrammetry resilient-struct call sites use it; ActivityKit has no resilient value. At immutable maintenance revision `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`, `tools/native-build-support` records the header as a Cargo input and adds its include dir to the target Clang command. The ordinary checkout keeps each small Cargo bridge and declarative build spec; `tools/releases/manifest-v1.tsv` pins the builder source revision. The host fixture checks alignment, alloc reject, and one destroy call; no run in this pass. No runtime dep from the helper

No generic `swiftcall` invoker, Swift type registry, Swift value copier, closure bridge, task entry, executor, or async adapter was added. No Apple API signature, deployment floor, framework link, Rust result, or C ABI changed. The opaque value init and destroy calls remain beside the capability-specific oracle that proves them.

## Evidence limits

The compiler and static link probes do not prove Simulator execution, physical-device behavior, Apple parity, or Xcode 27.x qualification. Link probes build and inspect example binaries only. The async lowering fixtures do not establish a supported public task-entry/resume contract.
