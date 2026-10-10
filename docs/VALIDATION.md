# Validation and Tooling

Use `cargo xtask` for non-validation shared checks from the repository root; install the pinned PATH tools before framework builds and validation. Run harness fixtures with the direct `cargo test` commands below. Reports go under `target/xtask/` by default and are build artifacts, not machine-specific source files.

```bash
cargo xtask toolchain-manifest --output target/xtask/toolchain-manifest.json
cargo xtask ios-build --simulator --release
cargo xtask ios-build --device --release
cargo xtask archive-smoke
cargo xtask no-std-check
cargo xtask no-std-link-probe
cargo xtask sdk-inventory --sdk all --output target/xtask/sdk-inventory-all.json
cargo xtask dependency-audit
cargo xtask abi-audit --output target/xtask/abi-audit.json
cargo xtask codegen-audit
cargo xtask linkage-audit --binary path/to/consumer
cargo xtask zero-swift-source
cargo xtask docs-check
tools/install-tools.sh --prefix target/ios-rust-tools
export PATH="$PWD/target/ios-rust-tools/bin:$PATH"
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --list
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-activitykit-status
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-homekit-identify-status
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --changed <explicit-base>
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --all --format json
cargo test -p parity-harness
cargo test -p bench-harness
```

## Shared native pilot validation

`ios-rust-validate` is limited to HomeKit identify status, photogrammetry status, AlarmKit status, and ActivityKit status. Each pilot keeps its existing feature, negative, public-import, symbol, availability, ownership, and compiler-oracle assertions; the installed runner owns command selection, target/SDK preflight, result formatting, and changed-file selection. `--explain ID` reports declared inputs, gate names, targets, SDKs, deployment floors, required and optional checks, expected public imports, ABI oracles, source guards, adapters, evidence limits, and effective commands. `--changed BASE` requires an explicit Git commit/ref, includes tracked, untracked, and deleted worktree paths, and expands shared-tool changes through the dependency graph; unknown dependencies select all pilots. CI and each pilot `check.sh` call the PATH executable directly. Required missing or invalid Python adapters report `error-blocked`, never pass; optional adapter failures remain visible in structured results without blocking unrelated required gates.

Human output and `--format json` distinguish `pass`, `fail`, `skipped`, and `error-blocked` gates with a reason. Apple SDK or Rust target absence skips only gates that need them and is never reported as a pass. Target compile, compiler ABI, link/import scan, simulator execution, and physical-device proof are separate evidence classes. The current link probes are inspected but not executed; simulator execution and physical-device runtime proof are reported as skipped. This harness does not implement application APIs or expand capabilities.

## What the checks prove

- `no-std-check` checks each portable crate with its default features and with `--no-default-features`, verifies `#![no_std]` in each crate root, and queries separate normal/dev/build Cargo trees with all features and targets. It rejects direct package IDs that appear in both normal and dev/build trees, and rejects internal workspace packages under `platform/` from the full normal dependency tree. The package list is maintained in `tools/xtask/src/main.rs`; add each new portable crate there. This does not infer third-party semantic intent, classify every transitive proc-macro/build-script role, or prove no `std` link

- `no-std-link-probe` selects the current 47-entry `PORTABLE_CRATES` registry and builds a transient `#![no_std]` static library with `--no-default-features`, a `#[panic_handler]`, `panic=abort`, and an explicit C `malloc`/`free` global allocator. Its C `main` invokes one selected public API check per crate, calls `framework_owned_buffer_destroy`, and requests/frees one aligned block. The macOS host artifact is linked, audited, and run. The same static library is built for `aarch64-apple-ios` and `aarch64-apple-ios-sim`, linked with a C harness into arm64 device and Simulator dylibs, and audited without execution. All three archives and linker maps are scanned for Rust `std` members; the Mach-O artifacts are checked for architecture/platform metadata, required symbols, unresolved Rust symbols, and imports. The exact dynamic import allowlist is `libSystem.B.dylib`; however, the last recorded successful link-probe report covers 45 packages (2026-10-08), not all 47 current registry entries
- `ios-ui` under `platform/ios/ios-ui` is a platform crate and stays outside the portable package list
- The host and both Apple target maps have separate archive-object evidence for 13 crates. Thirty-one API-only crates, including HDR playback eligibility, Bluetooth discovery, HealthKit authorization, cloud identity, web URL/navigation contracts, Game Center local-player status, ARKit world-tracking support, and Vision text-recognition revision support, have invoked checks in the Rust probe object without separate crate-object attribution; `framework-platform` is checked at compile time. This is selected-API link evidence, not full per-crate archive-object proof; the report keeps `all_portable_crates_no_std_linkage_proven` false. The last recorded 24-package run passed on 2026-10-08 with Rust 1.94.1, Xcode 26.6 (build 17F113), and iPhoneOS/iPhoneSimulator SDK 26.5. D21–D34 added twelve portable crates; the next probe covered 36 packages. The 45-package `no-std-link-probe` and `no-std-check` both passed locally on 2026-10-08; the latest 47-package `no-std-check` passed locally on 2026-10-09 with Rust 1.94.1. No link probe was run in that refresh, so link evidence remains 45 packages. The Apple artifacts are `target/xtask/no-std-link-probe/no-std-link-probe-ios-device.dylib` (minimum iOS 12.0) and `target/xtask/no-std-link-probe/no-std-link-probe-ios-sim.dylib` (minimum iOS Simulator 14.0); each is arm64, imports only `/usr/lib/libSystem.B.dylib`, and has no unresolved Rust-mangled symbol or `rust_eh_personality`. Their `nm -u` output includes `__Unwind_Resume` and common C/runtime imports resolved by `libSystem.B.dylib`. The target link uses explicit `-Wl,-undefined,error`, never `dynamic_lookup`; a target-local `rust_eh_personality` C fallback calls `abort()` and exists only as a fail-fast guard for this `panic=abort` fixture, not as an unwinding personality implementation. The Apple dylibs are not executed, so the evidence is link-only and does not establish panic or runtime behavior on a device or simulator. The explicit allocator path requests and frees one aligned block through `alloc::alloc`; allocation-backed collection APIs use an empty slab, so nonempty collection allocation is not exercised. A direct `cargo +1.94.1 rustc --locked -p framework-core -- --crate-type staticlib` probe had failed with `#[panic_handler] function required, but not found` and `unwinding panics are not supported without std`; `otool -L` alone cannot see static Rust `std`
- `ios-build` requires exactly one of `--simulator` or `--device` plus `--release`. It maps the flag to the `simulator` or `device` positional argument accepted by `examples/ios-minimal/build.sh`, which builds and bundles that minimal example in Release mode. This is an example build path, not proof of full-framework V1 support.
- `dependency-audit` saves Cargo feature, duplicate-version, and build-dependency graphs, with direct/transitive package-node counts and a count of enabled features named `std`. It is an inventory only: there is no dependency-growth baseline, Cargo metadata does not prove that every transitive dependency avoids `std`, and proc-macro target kinds are not classified.
- `sdk-inventory` records public framework header, module-map, and `.swiftinterface` paths plus declaration counts and heuristic flags for availability, Objective-C exposure, async/throws, generics, actor isolation, and protocol conformance. It is useful for drift triage, not a complete parser or public API/compliance review.
- `abi-audit` records Rust source declarations and the ABI version constants. It does not verify compiled C layouts, the final symbol table, the linked calling convention, Swift runtime provenance, or panic containment in a linked consumer.
- `linkage-audit` uses `otool -L` on a supplied Mach-O binary. The minimal example results below are scoped to those two artifacts; no general expected-import policy or unrelated-capability absence gate is claimed.
- `cargo xtask codegen-audit` emits optimized LLVM IR for the host and each installed iOS Rust target, then records IR paths and parsed bodies in `target/xtask/codegen-audit.json`. Its transient `#![no_std]` Rust-ABI probe declares no `extern "C"` functions and requires `OperationId::get` and `OperationId::new` plus `get` to lower to identity `i64` returns, and both `OperationId` and `Option<OperationId>` to use 8-byte layouts. It rejects call, allocation, atomic, and lock instructions in those probe functions, so there is no Rust or FFI call round trip in the audited bodies. This is structural evidence for one wrapper, not a benchmark, full-program LTO/link audit, general FFI ABI proof, or runtime/performance claim; compiler/LLVM upgrades can change IR and require review.
- `sh bindings/c/check.sh` compiles public headers as C11 and C++17, checks the recorded C layouts and static-library symbol set, then links and runs a C consumer. CI runs this check on macOS.
- `sh bindings/cpp/check.sh` formats and compiles the header-only C++17 layer, checks that consumers reference only the core C ABI symbols, links the ABI-version and RAII owner consumers against the real static library (including `framework_owned_buffer_destroy`), and checks move-only ownership with a test stub. CI runs this check on macOS.
- `sh bindings/c/check-secure-storage.sh` validates the opt-in Keychain C ABI: feature isolation, C11/C++17 headers, symbols, and host `UNSUPPORTED` stubs. On device and simulator, it links a C probe that calls read, store, and remove, then uses `otool -L` to enforce the exact direct import set `CoreFoundation`, `Security`, and `libSystem.B.dylib`; its `nm -u` scan rejects Swift, Objective-C, and network symbols. This import evidence applies to the probe, not arbitrary app links, and does not exercise a live Keychain; CI runs it on macOS
- `cargo check --locked -p ios-secure-storage --target aarch64-apple-ios`, `cargo check --locked -p ios-secure-storage --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-secure-storage --all-targets --target aarch64-apple-ios -- -D warnings`, and the corresponding simulator Clippy command compile and lint the Keychain backend on both Apple targets. These checks do not access a live Keychain or establish persistence, lock-state, or signed-app behavior; CI runs them on macOS.
- `cargo check --locked -p ios-files --target aarch64-apple-ios`, `cargo check --locked -p ios-files --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-files --all-targets --target aarch64-apple-ios -- -D warnings`, and the corresponding simulator Clippy command compile and lint the sandbox file backend and B17 Foundation file-coordination extension. These checks do not exercise a live app container, filesystem mutation, symlink behavior, runtime containment, provider coordination, or the documented concurrent native-directory-rename race; CI runs them on macOS.
- `cargo check --locked -p ios-preferences --target aarch64-apple-ios`, `cargo check --locked -p ios-preferences --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-preferences --all-targets --target aarch64-apple-ios -- -D warnings`, and the corresponding simulator Clippy command compile and lint the `NSUserDefaults` backend. These checks do not exercise live preference persistence, synchronization, or crash durability; CI runs them on macOS.
- `cargo check --locked -p ios-network --target aarch64-apple-ios`, `cargo check --locked -p ios-network --target aarch64-apple-ios-sim`, and target-specific all-target Clippy with `-D warnings` compile and lint the URLSession backend. `sh platform/ios/ios-network/check-link-imports.sh` builds and links a no-request probe for both targets; `otool -L` checks exact direct imports `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`, while `nm -u` rejects selected Swift/Python runtime and unrelated-capability symbol patterns. The probe binaries do not run. Hosted run `38075483431` passed both Xcode 27 device/Simulator checks, both strict Clippy checks, and the link/import gate on Xcode 27.0 / SDK 27.0. These checks do not exercise URLSession or establish HTTP parity.
- G12 locally passed `cargo +1.94.1 check --locked -p ios-connectivity --target aarch64-apple-ios`, the corresponding `aarch64-apple-ios-sim` check, all-target Clippy with `-D warnings` for both targets, and `sh platform/ios/ios-connectivity/check-link-imports.sh` on Xcode 26.6 build 17F113 / SDK 26.5. The arm64 device and Simulator probe executables import exactly Network.framework and `/usr/lib/libSystem.B.dylib`; the script's Swift/Python, Objective-C class/meta-class, and unrelated Network-symbol denylist passed. Hosted run `38075483431` passed the wired device/Simulator, Clippy, and link/import gates on Xcode 27.0 / SDK 27.0. The probe executables were not run. These are compile/link/import checks only; they do not establish path-change delivery, network availability, endpoint reachability, request success, or cancellation timing
- G13 locally passed device/Simulator `cargo check`, strict all-target Clippy, and `sh platform/ios/ios-data/check-link-imports.sh` on Rust 1.94.1, Xcode 26.6 build 17F113, and SDK 26.5. Both arm64 probe executables import exactly CoreFoundation.framework and `/usr/lib/libSystem.B.dylib`; the script's Swift/Python runtime, Objective-C class/meta-class, Security, mutable/no-copy data, string, and unrelated framework denylist passed. `vtool -show-build` reports device `LC_VERSION_MIN_IPHONEOS` 10.0 and Simulator `IOSSIMULATOR` minimum 14.0, both with SDK 26.5. Hosted run `38075483431` passed the wired device/Simulator, Clippy, and link/import gates on Xcode 27.0 / SDK 27.0. The probe executables were not run. These checks do not establish live app use, `NSData` runtime behavior, parity, allocation behavior under memory pressure, or performance
- G14 device/Simulator `cargo check`, strict all-target Clippy, and the corrected `sh platform/ios/ios-url/check-link-imports.sh` passed locally on Rust 1.94.1, Xcode 26.6 build 17F113, and SDK 26.5. Both arm64 probes import exactly Foundation.framework, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` confirms `_objc_alloc`, `_objc_getClass`, and `_objc_msgSend` plus retain/release and selector symbols, with no forbidden runtime or unrelated-capability imports. `strings` contains `NSString`, `NSURL`, and `URLWithString:encodingInvalidCharacters:` but not legacy `URLWithString:`. `vtool -show-build` reports iOS 17.0 minimum and SDK 26.5 for device and Simulator; the B20 API floor is separately iOS 17.0. The first script attempt failed because it required `_OBJC_CLASS_$_NSString`; the corrected gate checks dynamic Objective-C runtime symbols and Foundation class/selector strings. Hosted run `38075483431` passed the wired Xcode 27 device/Simulator and link/import gates. Probes were linked, not executed. These checks do not establish Foundation parser parity, acceptance of every D8 `Uri`, URL-open behavior, runtime availability under an incorrect app deployment target, or performance
- G15 locally passed locked device/Simulator `ios-ui` checks, strict all-target Clippy on both targets, and `sh platform/ios/ios-ui/check-link-imports.sh` on Rust 1.94.1 / Xcode 26.6 build 17F113 / SDK 26.5. Both arm64 probes import exactly CoreGraphics.framework and `/usr/lib/libSystem.B.dylib`; `dyld_info -imports` maps `_CGRectIntersection`, `_CGRectIsNull`, and `_CGRectIsEmpty` to CoreGraphics, with C/runtime allocation, pthread, dispatch, and unwind imports from libSystem. The link uses `-Wl,-dead_strip_dylibs` to remove autolinked UIKit, Foundation, CoreFoundation, and libobjc load commands with no symbol references from the geometry probe. C and Rust layout assertions pass for `CGFloat`, `CGPoint`, `CGSize`, and `CGRect`; C also checks `bool` size. `vtool -show-build` reports iOS minimum 12.0 for device and 14.0 for Simulator, SDK 26.5 for both. Hosted run `38075483431` passed the wired Xcode 27 device/Simulator, Clippy, and link/layout gates. Probes were built but not executed. These checks do not prove runtime geometry parity, visual behavior, or a general geometry API
- G16 device/Simulator `ios-media` checks, strict all-target Clippy, and the final link/import/layout script passed locally on Rust 1.94.1 / Xcode 26.6 build 17F113 / SDK 26.5. Both arm64 probes import exactly CoreMedia and `/usr/lib/libSystem.B.dylib`; `dyld_info -imports` maps `_CMTimeMake` to CoreMedia and all C/runtime symbols to libSystem. The final script uses `-Wl,-dead_strip_dylibs` to remove the unused CoreFoundation load command. C/Rust assertions verify `CMTime` size 24, alignment 4, and field offsets 0, 8, 12, and 16. `vtool` reports device minimum iOS 12.0 and Simulator minimum 14.0, SDK 26.5 for both. Hosted run `38075483431` passed the wired Xcode 27 device/Simulator, Clippy, and layout/link gates. Probes were built but not run. These checks do not establish live CoreMedia or AVFoundation use, playback, capture, runtime parity, or performance
- `sh bindings/c/check-notification-responses.sh` checks feature isolation, manifest-derived C11/C++17 response tags and view layout, exact view-field/status/output-slot/ownership manifest values, manifest/header/archive symbol parity, and host C/C++ static-library links for the opt-in response-value ABI. Manual review must still confirm that ownership text matches the implementation and guide. It does not execute C/C++ programs or prove response delivery or runtime behavior; CI runs it on macOS
- `sh bindings/c/check-ios-clipboard.sh` checks default/opt-in feature isolation, non-iOS stubs, C11/C++17 static-library links for host, iOS device, and simulator, strict Rust Clippy, and ABI symbols. Its linked device/simulator C consumers import exactly `Foundation`, `UIKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imports `libc++.1.dylib`. `nm -u` rejects share-only, Swift/Python, and unrelated capability symbols. The script does not execute consumers or exercise a live pasteboard or privacy UI; CI runs it on macOS
- `sh bindings/c/check-ios-share.sh` checks default/opt-in feature isolation, non-iOS stubs, C11/C++17 static-library links for host, iOS device, and simulator, strict Rust Clippy, and manifest-derived ABI tags, layouts, symbols, and direct-import allowlists. Linked C consumers import exactly `Foundation`, `UIKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imports `libc++.1.dylib`. `nm -u` checks linked probes and archives for clipboard, Swift/Python, and unrelated framework symbols. It does not execute consumers or present live share UI; CI runs it on macOS
- `sh bindings/c/check-ios-media-library-status.sh` validates F10's isolated feature graph, host/device/Simulator locked checks and strict Clippy, C11/C++17 consumers, exported symbol, exact Foundation/MediaPlayer/libSystem/libobjc imports (plus libc++ for C++), selector strings, and device/Simulator minos 10.0/14.0. The C/C++ probes are not executed; the B71 API floor remains iOS 9.3. CI runs this gate on macOS
- `sh bindings/c/check-ios-spritekit.sh` validates F11's isolated feature graph, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, four F11 symbols, and exact CoreFoundation/Foundation/SpriteKit/UIKit/libSystem/libobjc imports (plus libc++ for C++). It does not execute consumer/probe binaries; B70’s API floor remains iOS 7.0, with probe minos 12.0/device and 14.0/Simulator. CI runs this gate on macOS
- `sh bindings/c/check-ios-call-observer.sh` validates F12 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, one exported snapshot function, and exact CallKit/Foundation/libSystem/libobjc imports (plus libc++ for C++). It does not execute consumers/probes; B72's API floor remains iOS 10.0, with C ABI probe minos 12.0/device and 14.0/Simulator. CI runs this gate on macOS
- `sh bindings/c/check-ios-maps.sh` validates F13 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, three exported geometry functions, and exact MapKit/libSystem imports (plus libc++ for C++). It does not execute consumers/probes; B73's API floor remains iOS 4.0, with C ABI probe minos 12.0/device and 14.0/Simulator. CI runs this gate on macOS
- `sh bindings/c/check-ios-classkit-deep-link.sh` validates F14 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, one exported marker function, exact ClassKit/Foundation/libSystem/libobjc imports (plus libc++ for C++), selector and forbidden-data checks. It does not execute consumers/probes; B74's API floor is iOS 11.3, with probe minos 11.3/device and 14.0/Simulator. CI runs this gate on macOS
- `sh bindings/c/check-ios-file-provider.sh` validates F16 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, three exported functions, exact FileProvider/Foundation/libSystem/libobjc imports (plus libc++ for C++), owned-buffer creator metadata, and device/Simulator minos 11.0/14.0. It does not execute consumers or probes; CI runs this gate on macOS
- `sh bindings/c/check-ios-vision.sh` validates F17 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, one exported function, exact Vision/Foundation/libSystem/libobjc imports (plus libc++ for C++), selector strings, forbidden-scope symbols, and minos 13.0/14.0. Its static gate checks aligned writable output storage through call return, caller protection from unsynchronized access, no output-pointer retention, and null behavior. It does not execute consumers or probes; CI runs this gate on macOS
- `sh bindings/c/check-ios-proximity-reader.sh` validates F18 feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, one exported function, exact ProximityReader/libSystem imports (plus libc++ for C++), two public Swift symbol imports without Swift/Objective-C runtime imports, and minos 15.4. Its static gate checks aligned writable output storage through call return, caller protection from unsynchronized access, no output-pointer retention, and null behavior. It does not execute consumers or probes; CI runs this gate on macOS
- `cargo check --locked -p ios-location --target aarch64-apple-ios` and `cargo check --locked -p ios-location --target aarch64-apple-ios-sim` compile the Core Location backend for device and simulator targets. `cargo clippy --locked -p ios-location --all-targets --target aarch64-apple-ios -- -D warnings` and `cargo clippy --locked -p ios-location --all-targets --target aarch64-apple-ios-sim -- -D warnings` apply the same target-specific lint gate. `sh platform/ios/ios-location/check-link-imports.sh` checks exact Core Location feature trees and a Rust source guard for required one-shot/foreground-authorization calls, confines `requestWhenInUseAuthorization()` to the explicit authorization-request path, and rejects excluded continuous, region, significant-change, visit, heading, beacon-ranging, Always-authorization, temporary-accuracy, and background-location APIs before it links device and Simulator probes. It uses `otool -L` to require exactly `CoreLocation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; its `nm -u` scan rejects Swift/Python runtime and selected unrelated capability symbols. The source guard does not constrain host calls through the borrowed native-manager escape. These checks do not execute the probes or exercise live permission UI, GPS delivery, fix quality or freshness, cancellation races, or device/simulator runtime behavior.
- `cargo check --locked -p ios-motion --target aarch64-apple-ios` and `cargo check --locked -p ios-motion --target aarch64-apple-ios-sim` compile the Core Motion backend for device and simulator targets. `cargo clippy --locked -p ios-motion --all-targets --target aarch64-apple-ios -- -D warnings` and the corresponding simulator command apply strict target-specific lint gates. These checks do not establish sensor delivery, callback/cancellation timing, privacy behavior, or physical-device availability.
- `cargo check --locked -p ios-auth --target aarch64-apple-ios` and `cargo check --locked -p ios-auth --target aarch64-apple-ios-sim` compile the LocalAuthentication and App Tracking Transparency status backends for device and simulator targets. `cargo clippy --locked -p ios-auth --all-targets --target aarch64-apple-ios -- -D warnings` and the corresponding simulator command apply strict target-specific lint gates.
- `sh platform/ios/ios-auth/check-link-imports.sh` builds a device and simulator probe binary and checks each Mach-O import list with `otool -L` against the exact allowlist `AppTrackingTransparency.framework`, `Foundation.framework`, `LocalAuthentication.framework`, `libSystem.B.dylib`, and `libobjc.A.dylib`. It scans undefined symbols with `nm -u` and rejects Swift runtime or Keychain symbols. It does not execute either binary or exercise a live authentication prompt, biometric sensor, enrollment state, or host-app plist
- `sh platform/ios/ios-auth/check.sh` runs the shared `framework-auth` portable status tests/no-default check/strict Clippy and device/Simulator `ios-auth` checks/strict Clippy. It does not read live ATT status, display an ATT prompt, access an advertising identifier, or track
- `sh platform/ios/ios-metal/check.sh` runs `framework-metal` tests/no-default check/strict Clippy/rustdoc plus device and Simulator `ios-metal` checks/strict Clippy, a Release link/import audit, docs check, and zero-Swift-source gate. The probes import exactly Metal, Foundation, `libobjc.A.dylib`, and `libSystem.B.dylib`, with `_MTLCreateSystemDefaultDevice` and `_objc_release`; they are not executed and do not submit GPU work or establish device performance
- `cargo check --locked -p ios-notifications --target aarch64-apple-ios`, `cargo check --locked -p ios-notifications --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the B4 local-notification backend for device and simulator targets. These are G5 CI target-specific check and Clippy gates. `sh platform/ios/ios-notifications/check-link-imports.sh` is now wired into the macOS CI workflow and passed locally on Rust 1.94.1, Xcode 26.6 (17F113), and iPhoneOS SDK 26.5 after root refreshed `Cargo.lock`: both target import lists were exactly `Foundation`, `UserNotifications`, `libSystem.B.dylib`, and `libobjc.A.dylib`, and both `nm -u` denylist scans passed. The first script attempt exited 101 before either target build because `--locked` required the shared `Cargo.lock` refresh; after that refresh the rerun passed. The linked binaries were inspected but not executed, and no passing CI workflow run is recorded. Neither the CI gates nor the import check request permission, deliver or observe a notification, or establish runtime behavior
- `cargo check --locked -p ios-notification-responses --target aarch64-apple-ios`, `cargo check --locked -p ios-notification-responses --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-notification-responses --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-notification-responses --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the separate B12 response bridge for device and simulator targets. These checks do not exercise a live response callback, notification delivery, permission prompt, or runtime behavior.
- `cargo check --locked -p ios-sharing --target aarch64-apple-ios`, `cargo check --locked -p ios-sharing --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the clipboard and outgoing-share backends for device and simulator targets. These prove compile/lint only; they do not access a live pasteboard, show or inspect a privacy prompt, present share UI, observe an activity result, or prove callback/drop-race behavior.
- `cargo check --locked -p ios-accessibility --target aarch64-apple-ios`, `cargo check --locked -p ios-accessibility --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the UIKit accessibility metadata backend for device and simulator targets. These are compile/lint checks only: they do not run an app, exercise live VoiceOver, verify focus movement or announcement delivery, or prove assistive-technology behavior or accessibility UX.
- `cargo check --locked -p ios-presentation --target aarch64-apple-ios`, `cargo check --locked -p ios-presentation --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the UIKit acknowledgement-alert crate for device and simulator targets. These are compile/lint checks only: they do not run an app, present or dismiss a live alert, or prove user interaction.
- `cargo check --locked -p ios-resources --target aarch64-apple-ios`, `cargo check --locked -p ios-resources --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-resources --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-resources --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the main-bundle resource backend for device and simulator targets. These are compile/lint checks only: they do not run an app, perform a live bundle-resource lookup, or prove resource packaging or runtime behavior; they also do not establish symlink containment, localized lookup, or asset-catalog access.
- `cargo check --locked -p ios-browser --target aarch64-apple-ios`, `cargo check --locked -p ios-browser --target aarch64-apple-ios-sim`, `cargo clippy --locked -p ios-browser --all-targets --target aarch64-apple-ios -- -D warnings`, and `cargo clippy --locked -p ios-browser --all-targets --target aarch64-apple-ios-sim -- -D warnings` compile and lint the external HTTPS URL-handler backend for device and simulator targets. These are compile/lint checks only: they do not make a live URL-handler call, launch a browser, send a network request, prove page-load behavior, or establish app/browser runtime behavior.
- `sh interop/swift-abi-core/tests/check-scalar-swiftcall.sh` checks one synthetic scalar Swift ABI call: a Rust host caller links and runs through a Clang `swiftcall` shim; device/simulator Swift, Clang, and `no_std` Rust objects compile only. The script writes transient Swift input outside the checkout; CI runs it on macOS.
- `sh interop/swift-abi-core/tests/check-retained-ownership.sh` runs a transient Swift class fixture through the opt-in Rust `SwiftRetained` wrapper for 64 exact deinit cycles, verifies compiler-emitted `swift_retain`/`swift_release` declarations and one host `libswiftCore` load, checks that default features omit the runtime, and compiles Swift oracle objects plus Rust crates for device/simulator. Only the host class-lifetime path runs; iOS target evidence is compile-only. CI runs it on macOS.
- `sh interop/swift-abi-core/tests/check-swift-string-cxx.sh` builds a temporary Swift `String` C++ API and fixed-width C shim, then checks six exact host UTF-8 fixtures. Device/simulator Swift, C++, and no_std Rust results are compile-only; the opt-in host proof links one Swift runtime. This is not a production ABI or Translation claim.
- `sh interop/swift-abi-core/tests/check-swift-optional-cxx.sh` checks generated `String?` C++ bindings with fixed-width optional-state and byte fields for `none` and six `some` UTF-8 fixtures. Device/simulator Swift, C++, and no_std Rust results are compile-only; the opt-in host proof links one Swift runtime. This is not a production Optional adapter or Translation claim.
- `sh interop/swift-abi-core/tests/check-swift-async-cxx.sh` records whether the public generated C++ header exposes synchronous, `async`, and `async throws` Swift controls. The active compiler omits both async declarations, so the script stops before a C++/Rust call path; it proves no async ABI, executor, cancellation, completion, or target behavior.
- `sh interop/swift-abi-core/tests/check-swift-async-thunk.sh` compares compiler-derived Swift `async -> Int32` and `async throws -> Int32` entry signatures with Clang `swiftasynccall` forwarding on host, device, and simulator. It proves a bounded calling-convention lowering match; target evidence is compile/object only, and the script stops before invocation because task creation, context lifetime, resume ownership, executor behavior, and error delivery are not established as a public C/Rust contract.
- `sh interop/swift-abi-core/tests/check-swift-async-runtime-entry.sh` audits installed Swift concurrency interfaces, C headers/module maps, export stubs, compiler-generated task/context paths, and generated C++ headers. It does not call runtime task symbols or invoke an async entry; device/simulator evidence is compiler IR/SIL only. Its Xcode 26.6 result is a scoped no-go for a supported public C/C++ task-entry contract, not a claim that exported internal symbols do not exist.
- The manual C7 [App Intents Stage 0 audit](swift-abi/APP_INTENTS_STAGE0.md) records a temporary Release simulator `xcodebuild` and the normal metadata extraction steps. It confirms the observed `Metadata.appintents` output only; processor commands, file lists, and metadata encoding are implementation details, not stable Rust/C inputs. Stage 1 is unsupported on the audited Xcode 26.6 toolchain; no intent invocation, install, or runtime behavior was tested.
- `zero-swift-source` scans all `.swift` files in the checkout except those under root `.git/` and `target/`, with no Git-state filter; keep Swift oracle input outside the checkout.

The parity and benchmark packages provide harness unit fixtures only; they do not register real Apple/reference suites or framework workloads and produce no parity/performance evidence. `cargo xtask parity` remains unavailable until an Apple reference adapter and Rust candidate suite are registered. Physical-device performance remains unavailable. `cargo xtask codegen-audit` supplies only the bounded structural `OperationId` IR evidence described above; broad optimized-codegen checks remain unavailable. `archive-smoke` runs the shared `ios-minimal` Xcode scheme with the standard `xcodebuild archive` action, disables code signing, and verifies the resulting app bundle, plist, imports, and absence of `.swift` files in the archive. It is an unsigned packaging check, not a signing, provisioning, export, installation, or runtime check.

## Current host baseline

The inspected host reports macOS 26.6.2 (build 25G83), Xcode 26.6 (build 17F113), iPhoneOS and iPhoneSimulator SDK 26.5, Swift 6.3.3, Apple Clang 21.0.0 (LLVM build `clang-2100.1.1.101`), and Rust 1.94.1. The plan requires Xcode 27.x, so this host does **not** satisfy its Xcode baseline. `toolchain-manifest` records the mismatch and emits a warning; it does not fail the host Rust checks or relabel Xcode 26.6 as 27.x.

Rust 1.94.1 has these targets installed on this host: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-darwin`. The repository has no full framework Xcode project or iOS framework crate, and no shared deployment target is defined; the minimal example script sets `IPHONEOS_DEPLOYMENT_TARGET=17.0` for its own build.

## iOS minimal example results (2026-10-07–2026-10-08)

On the recorded Xcode 26.6 host, `cargo xtask archive-smoke` passed on 2026-10-07 and twice on 2026-10-08; the latest pass follows the adaptive-color and callback-smoke example changes. It built the Release device executable with Cargo, then ran the standard Xcode archive action using the `ios-minimal` shared scheme, `-destination 'generic/platform=iOS'`, and `CODE_SIGNING_ALLOWED=NO`. The archive is at `target/ios-minimal/archive/ios-minimal.xcarchive`; its app is `target/ios-minimal/archive/ios-minimal.xcarchive/Products/Applications/ios-minimal.app`. `plutil -lint` passed for the archive and app `Info.plist` files; `codesign -dv` reported `code object is not signed at all`. The archived executable imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`, with no Swift or Python runtime imports. The archive contains no `.swift` source.

On the same host, both `cargo xtask ios-build --simulator --release` and `cargo xtask ios-build --device --release` passed on 2026-10-07 and were rerun successfully on 2026-10-08 after the adaptive-color change. They produced unsigned bundles at `target/ios-minimal/simulator/ios-minimal.app` and `target/ios-minimal/device/ios-minimal.app`. Both bundle executables imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`; neither imported the Swift or Python runtime. `plutil -lint` passed for `target/ios-minimal/simulator/ios-minimal.app/Info.plist` and `target/ios-minimal/device/ios-minimal.app/Info.plist`.

The archive emitted Xcode warnings that all interface orientations must be supported unless full-screen is required, and that a launch configuration/storyboard must be provided unless full-screen is required. These results do not meet the plan's Xcode 27.x baseline. A separate x86_64 simulator build launched on iOS 18.0 and displayed the label/button in light and dark appearance. With `--exercise-rust-button-callback`, the app sent `UIControlEvents::TouchUpInside` through `UIControl::sendActionsForControlEvents` and the Rust callback changed the label; this was programmatic dispatch, not a user touch, and does not validate arm64 simulator execution. No archive signing/provisioning, archive export, arm64 simulator launch, device installation, or physical-device validation was performed. The archive result is unsigned packaging evidence from this Xcode 26.6 host only.

## CI checks

macOS CI is configured to run `cargo xtask no-std-link-probe` after `no-std-check`; the command selects all 47 current registry entries. No passing CI workflow run is recorded for this configuration, and the latest successful local link-probe report covers 45 packages. The fresh 47-package local `no-std-check` passed on 2026-10-09. The probe does not establish full-application no_std linkage, every API path, or device/simulator runtime behavior

G41 D42/B47 `ios-media` other-audio package gate is also wired in macOS CI. It compiles and lints
device and Simulator targets and audits the bounded AVAudioSession imports/selectors; probes are
not executed, and no live audio state is claimed

G42 `ios-playback/check.sh` and G43 `ios-message-ui-support/scripts/check.sh` plus
`ios-shared-with-you-support/scripts/check.sh` are wired in macOS CI and passed locally on Xcode 26.6 /
SDK 26.5. They compile and lint the bounded packages and link Release probes with exact framework
imports and deployment floors. Probes are not executed; no live HDR, mail/text status, account,
highlight, or collaboration behavior is claimed, and no CI workflow run is recorded.

G44 `crates/framework-payments/scripts/check.sh` is wired in macOS CI and passed locally on Rust
1.94.1 / Xcode 26.6 / SDK 26.5. It compiles and lints host, device, and Simulator targets, checks
rustdoc and exact PassKit/Foundation/runtime imports, and builds—but does not execute—the link probes.
No live Apple Pay status, card, merchant, or transaction behavior is claimed.

G45 `platform/ios/ios-media/check.sh` is wired in macOS CI and passed locally on Rust 1.94.1 /
Xcode 26.6 / SDK 26.5. It checks the portable FourCC/status values and iOS device/Simulator
compilation, strict Clippy, rustdoc, and that only the required VideoToolbox binding features are
enabled. No tests, link probes, live codec query, or device behavior were run.

G46 `platform/ios/ios-cloud/check-cloudkit.sh` is wired in macOS CI and passed locally on Rust
1.94.1 / Xcode 26.6 / SDK 26.5. It checks the portable no-default contract, device/Simulator
compilation, strict Clippy, and CloudKit/Foundation/libSystem/libobjc imports. Both link probes were
built but not executed; no live account query, entitlement validation, data access, or CI workflow
run is claimed. See [the focused G46 plan](../PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md).

G47 `platform/ios/ios-safety/scripts/check.sh` passed in its isolated worktree on Rust 1.94.1,
Xcode 26.6, and iPhoneOS SDK 26.5. Host/device/Simulator checks, strict Clippy, rustdoc, and
SafetyKit/Foundation/libSystem/libobjc import audits passed; probes were built, not executed. The
getter-specific entitlement requirement and live device result remain unverified. See the
[focused G47 plan](../PLAN_VALIDATION_IOS_SAFETYKIT.md).

G48 `platform/ios/ios-maps/check.sh` passed in the integrated checkout after its allowlist was
updated for the observed Foundation import. Portable no-default check/Clippy/docs, host/device/
Simulator check/Clippy, feature review, exact ARKit/Foundation/libSystem/libobjc imports, selector/
symbol/string filters, and device/Simulator minos 12.0/14.0 passed. Probes were not executed; no
runtime support query, camera access, or tracking result is claimed. See the
[focused G48 plan](../PLAN_VALIDATION_IOS_ARKIT.md); the same package gate now also validates the D77 MapKit geometry slice, whose details are in [G68](../PLAN_VALIDATION_IOS_MAPKIT.md).

G49 `platform/ios/ios-game/check.sh` passed portable no-default check/strict Clippy/docs, iOS
device/Simulator check/strict Clippy, and GameKit/Foundation import gates in its isolated worktree.
Link probes were built, not executed; no signed entitlement, live player state, or authentication UI
behavior is claimed. See the [focused G49 plan](../PLAN_VALIDATION_IOS_GAMEKIT.md).

G50 `platform/ios/ios-core-ml-status/scripts/check.sh` passed in the integrated checkout on Rust
1.94.1 / Xcode 26.6 / iPhoneOS SDK 26.5. Host/device/Simulator checks, strict Clippy, rustdoc, and
CoreML/Foundation/libSystem/libobjc link/import/symbol audits passed; probes were built, not
executed. No live compute-device list, model compatibility, or prediction behavior is claimed. See
the [focused G50 plan](../PLAN_VALIDATION_IOS_COREML.md).

G51 `platform/ios/ios-vision/check.sh` passed in the integrated checkout. Portable no-default check,
strict Clippy/docs, iOS host/device/Simulator checks, strict Clippy, feature-surface gates, and exact
Vision/Foundation/libobjc/libSystem imports passed. Device/Simulator probes were built, not
executed; no image or recognition was performed. See the [focused G51 plan](../PLAN_VALIDATION_IOS_VISION.md).

G52 `platform/ios/ios-speech-status/scripts/check.sh` passed in the integrated checkout. Host,
device, and Simulator checks, strict Clippy, rustdoc, source-surface checks, exact Speech/Foundation/
libobjc/libSystem imports, docs, and zero-Swift gates passed. Probes were built, not executed; no live
authorization, prompt, audio, or recognition behavior is claimed. See the [focused G52 plan](../PLAN_VALIDATION_IOS_SPEECH_STATUS.md).

G53 `platform/ios/ios-natural-language-status/scripts/check.sh` passed in the integrated checkout.
Host, device, and Simulator checks, strict Clippy, rustdoc, source-surface checks, exact
NaturalLanguage/Foundation/libobjc/libSystem imports, docs, and zero-Swift gates passed. Probes were
built, not executed; no live asset state, model load, or vector operation is claimed. See the
[focused G53 plan](../PLAN_VALIDATION_IOS_NATURALLANGUAGE_STATUS.md).

G54 `platform/ios/ios-storekit-status/scripts/check.sh` passed in the integrated checkout. Host,
device, and Simulator checks, strict Clippy, rustdoc, source/feature guards, exact StoreKit/Foundation/
libobjc/libSystem imports, docs, and zero-Swift gates passed. Probes were built, not executed; no live
purchase ability, product availability, account state, or transaction behavior is claimed. See the
[focused G54 plan](../PLAN_VALIDATION_IOS_STOREKIT_STATUS.md).

G55 `platform/ios/ios-roomplan/check.sh` passed in the integrated checkout. Portable `no_std`,
host/device/arm64 Simulator checks, strict Clippy, rustdoc, the compiler-oracle `swiftcall` audit,
and exact RoomPlan/libSystem Release imports passed. The linker oracle verified the
`swift_context`/`swiftself` argument. Probes were built and inspected, not executed; no live device
query or scan is claimed. See the [focused G55 plan](../PLAN_VALIDATION_IOS_ROOMPLAN.md).

G56 `platform/ios/ios-camera-device-status/check.sh` passed in the isolated worktree and integrated
checkout. Host/device/Simulator checks, strict Clippy, rustdoc, source guards, exact
AVFoundation/Foundation/libobjc/libSystem imports, docs, zero-Swift, and diff gates are included.
Probes were built, not executed; no live camera, authorization, or capture behavior is claimed. See
the [focused G56 plan](../PLAN_VALIDATION_IOS_CAMERA_DEVICE_STATUS.md).

G57 `platform/ios/ios-storekit2-status/check.sh` passed in the isolated worktree and integrated
checkout. Host/device/Simulator checks, strict Clippy, rustdoc, transient Swift/Clang
IR comparison, SDK symbol verification, exact weak StoreKit/libSystem imports, docs, zero-Swift,
and diff gates passed. Probes were not executed; no live payment query or purchase behavior is
claimed. See the [focused G57 plan](../PLAN_VALIDATION_IOS_STOREKIT2_STATUS.md).

G58 `ios-safari` device/Simulator checks and strict Clippy plus
`platform/ios/ios-safari/check-link-imports.sh` passed in the isolated worktree and integrated
checkout. The release probes import Foundation, SafariServices, UIKit, libSystem, and libobjc;
their iOS 10.0 device and iOS 14.0 Simulator deployment floors and class/selector/Objective-C
metadata were inspected. Probes were not executed; no controller presentation, URL request, page
load, or browser parity is claimed. See the [focused G58 plan](../PLAN_VALIDATION_IOS_SAFARI.md).

G59 `ios-accelerate` locked device/Simulator checks, strict Clippy, and
`platform/ios/ios-accelerate/check-link-imports.sh` passed in the isolated validation workspace and
integrated checkout. Release probes import only Accelerate and libSystem, retain `_vDSP_vadd`, and
report iOS 10.0 device and iOS 14.0 Simulator link floors. The iOS SDK declares the API from iOS
4.0. Probes were not executed; no numerical result, parity, live-device behavior, or performance is
claimed. See the [focused G59 plan](../PLAN_VALIDATION_IOS_ACCELERATE.md).

G60 `ios-crypto` locked device/Simulator checks, strict Clippy, and
`platform/ios/ios-crypto/check-link-imports.sh` passed in the integrated checkout. Release probes
import only `libSystem.B.dylib`, retain `_CC_SHA256`, and report iOS 10.0 device and iOS 14.0
Simulator link floors; the inspected SDK header marks the API available from iOS 2.0. Probes were
built and inspected, not executed; no digest parity, security review, certification, live-device
behavior, or performance is claimed. See the [focused G60 plan](../PLAN_VALIDATION_IOS_CRYPTO.md).

After D55/B60 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy,
`xtask docs-check`, formatting, `git diff --check`, and dependency audit passed. No tests were run in
this pass.

After D56/B61 integration, locked workspace `cargo check`, the 45-crate `no-std-check`, and the
45-crate `no-std-link-probe` passed. The host probe exited 0; arm64 device and Simulator dylibs
linked and passed audits but were not executed. The report does not claim aggregate no-std linkage
proof for every selected API: 31 API-only crate calls lack separate object attribution.

After D57/B62 integration, locked workspace `cargo check`, strict all-target/all-feature Clippy,
the focused camera-device gate, docs-check, formatting, `git diff --check`, and dependency audit
passed. The 45-crate portable set did not change. No tests or live camera operations were run.

After D58/B63 integration, locked workspace `cargo check`, strict all-target/all-feature Clippy,
the focused StoreKit 2 gate, docs-check, formatting, `git diff --check`, dependency audit, and
capability JSON parsing passed. `cargo test --locked -p framework-sharing` passed 11 tests across
the clipboard and outgoing-share contracts. No full workspace tests, live purchase query, or
device/Simulator probe execution occurred. The portable set remains 45 crates.

After B64 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy,
the focused SafariServices device/Simulator gates and link-import audit, docs-check, formatting,
`git diff --check`, dependency audit, and capability JSON parsing passed. No tests or live Safari
controller, URL, or page actions were run; the portable set remains 45 crates.

After D59/B65 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy,
the focused Accelerate device/Simulator gates and link-import audit, rustdoc, docs-check, formatting,
`git diff --check`, dependency audit, and capability JSON parsing passed. No tests or link probes ran
as executables; portable counts remain 34 implemented and 20 partial across 45 crates.

After D60/B66 root integration, locked package device/Simulator checks, strict Clippy, the focused
CommonCrypto link/import audit, docs-check, formatting, zero-Swift-source, and `git diff --check`
passed. No tests or link probes ran as executables. The portable set remains 45 crates; row coverage
at this integration point is 72/113 `B` partial and 41 `X`, with 30 generic gaps and 11 specific
gaps.

After D61/B67 root integration, `sh platform/ios/ios-modelio-status/check.sh` passed: locked host,
device, and Simulator checks; strict Clippy; `MDLAsset` feature isolation; exact Release imports;
and iOS rustdoc. The imports were Foundation, ModelIO, `libSystem.B.dylib`, and `libobjc.A.dylib`;
probe minos was 10.0/device and 14.0/Simulator. Probes were inspected but not executed; no file
parse or runtime claim is made.

After D62/B68 root integration, the MPS package format, host/device/Simulator checks, strict
Clippy, rustdoc, and `MPSCore` feature-isolation gate passed, as did
`sh platform/ios/ios-mps-status/check-link-imports.sh`. Exact imports were Foundation, Metal,
MetalPerformanceShaders, `libSystem.B.dylib`, and `libobjc.A.dylib`; probe minos was 12.2/device and
14.0/Simulator. Probes were inspected but not executed; no GPU work or MPS operation support is
claimed. At that integration point, the manifest count was 74/113 `B` partial and 39 `X`, with 28 generic gaps and 11
specific gaps; the portable set remains 45 crates.

After D63/B69 root integration, the portable key-support and iOS Security package check scripts and
the exact device/Simulator link-import audit passed. Imports were CoreFoundation, Security, and
`libSystem.B.dylib`; symbols were `SecKeyCreateWithData` and
`SecKeyIsAlgorithmSupported`; probe minos was 10.0/device and 14.0/Simulator. Probes were
inspected but not executed. At that integration point, the manifest was 75/113 `B` partial and 38 `X`, with 28 generic gaps
and 10 specific gaps; the portable set is 46 crates (34 implemented and 21 partial contracts).
At this checkpoint the 46-crate `no-std-check` was pending the full integration audit; it was later
superseded by a passing 47-crate `no-std-check` on 2026-10-09. The last successful
`no-std-link-probe` report still covers 45 crates. No tests, live key query, signature verification,
Keychain/Secure Enclave behavior, or parity are claimed.

After D65/B71 root manifest and guide integration, the MediaPlayer package's locked host/device/Simulator
checks, strict Clippy, iOS rustdoc, formatting, shell syntax, and feature-isolation gates passed
in the integrated workspace. The manifest is 76/113 `B` partial and 37 `X`, with 27 generic and
10 specific gaps; no permission query, prompt, item read, service request, or playback was run.

After D64/B70 root integration, `sh platform/ios/ios-spritekit/check.sh` and
`sh platform/ios/ios-spritekit/check-link-imports.sh` passed on Rust 1.94.1, Xcode 26.6 build
17F113, and iOS SDK 26.5. Device and Simulator link probes were built and inspected, not executed.
The public API floor is iOS 7.0; probe `minos` is 12.0/device and 14.0/Simulator. F10's
`sh bindings/c/check-ios-media-library-status.sh` also passed in the integrated workspace after the
offline Cargo lock refresh. At that D64/B70 checkpoint, the manifest was 77/113 `B` partial and 36 `X`, with 20 generic
gaps and 16 specific scope/toolchain gaps; the portable set had 47 crates (35 implemented and 21
partial contracts). At this checkpoint the 47-crate `no-std-check` was pending; it later passed for
the current registry on 2026-10-09. The last successful `no-std-link-probe` report still covers 45
crates. D67, D69, D71, D74, D75, and D78 are feasibility-only audits for FamilyControls, Foundation
Models, DeviceActivity, AdAttributionKit/AdServices, Thread, and DockKit; rows 074, 067, 075, 089,
045, and 097 remain `X`. F11's `sh bindings/c/check-ios-spritekit.sh` passed after the offline Cargo lock refresh; it verified the opaque-handle C ABI, locked host/device/Simulator checks and strict Clippy, C11/C++17 link shape, exact imports, and deployment metadata. C/C++ consumer and target probe binaries were not executed; no SpriteKit scene or rendering behavior was exercised.

At the F14 checkpoint, the canonical matrix was 80/113 `B` partial and
33 `X` (9 generic missing-facade/backend gaps, 24 specific scope/toolchain gaps). The portable
roster remains 47 packages with 35 implemented and 22 partial contracts. The CallKit, ClassKit, and
MapKit package gates and all three new C ABI gates passed. Device/Simulator probes and C/C++ consumers
were inspected but not executed. No tests, live call/activity snapshots, Schoolwork flow, native map
projection, or MapKit distance parity were exercised. Xcode 26.6 build 17F113 / iOS SDK 26.5 remains
below the Xcode 27.x planning baseline.

G78 records D85/B75: the FileProvider package check, device/Simulator compile, strict Clippy, rustdoc, docs-check, and link/import audit passed; probes were inspected, not executed. Row 099 is `B` for registered-domain presence only. G79 records D86 ExtensionKit/Foundation, G80 records D87 ContactProvider, G82 records D88 BrowserEngineKit, G83 records D89 ManagedApp/Distribution, G84 records D90 MarketplaceKit, G85 records D91 MatterSupport, G86 records D92 SecureElementCredential, G87 records D93 CarKey, G88 records D94 ProximityReader, G89 records D95 LockedCameraCapture, G91 records D96 extension metadata, G93 records D97 WidgetKit, G94 records D98 ActivityKit, and G95 records D99 App Intents. These feasibility audits ran no live flows. G81 F15 Location and G90 F16 FileProvider C ABI gates passed, including F15 pointer-contract assertions; B5 device and Simulator feature-tree checks passed with exactly `CLLocation`, `CLLocationManager`, and `CLLocationManagerDelegate`; probes were inspected, not executed. G92 B76 ProximityReader host/device/Simulator checks, strict Clippy, rustdoc, Swift/Clang ABI oracles, and Release link/import/deployment gates passed; probes were not executed. G96 B77 host/device/Simulator compilation, strict Clippy, rustdoc, feature closure, and Foundation link/import gates passed; its probe was not executed. G97 B78 device/Simulator checks, strict Clippy, rustdoc, and AuthenticationServices link/import gates passed; no credential query or linked-probe execution occurred. G98 F19 and G99 F20 C ABI gates passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact imports, deployment floors, and static aligned-output lifetime, concurrency, range, and overlap assertions; their consumers/probes were not executed. G100 B79 host/device/Simulator checks, strict Clippy, rustdoc, feature closure, and NetworkExtension/Foundation link/import gates passed; probes were inspected, not executed, and no entitled VPN query ran. G101 F21 static and native link/import gates passed for C11/C++17 device/Simulator consumers, imports, Objective-C/block symbols, export, forbidden symbols, and minos 13.0/14.0; consumers were not executed. G102 F22's opt-in C ABI gate passed host/device/Simulator feature isolation, strict Clippy, C11/C++17 links, exact Accelerate/libSystem imports, `_vDSP_vadd`, and minos 10.0/14.0; consumers and probes were not executed. G103 B4’s UserNotifications device/Simulator link/import gate passed exact framework imports and the undefined-symbol denylist; it is wired in macOS CI, but no passing workflow run is recorded, and no notification state query or linked probe was executed. G104 F23’s static C11/C++17 and host/device/Simulator feature, Clippy, Release-build, link/import, export, forbidden-symbol, and minos gates passed. It is wired in macOS CI, but no passing workflow run is recorded; consumers and probes were not executed. G105 F24’s host/device/Simulator feature, strict-Clippy, Release-build, C11/C++17 link/import, symbol, export, forbidden-symbol, and minos gates passed. It is wired in macOS CI, but no passing workflow run is recorded; consumers and probes were not executed. Rows 021, 098, and 113 are `B` for B78's entitlement-scoped credential-state query, B79's Personal VPN profile-status query, and B77's one-key extension metadata read, respectively; rows 100–107 and 109–112 remain `X`. Row 108 is `B` for the device-model predicate only. The current matrix is 85/113 `B` (75.2% row coverage) and 28 `X`, all with specific scope/API/toolchain gaps; the portable roster is 58 contracts (36 implemented, 22 partial). No tests or linked C/C++ consumers were executed.


F25 `sh bindings/c/check-ios-videotoolbox.sh` and `sh bindings/c/check-ios-videotoolbox-link.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact VideoToolbox/libSystem device and Simulator imports, export parity, and minos 11.0/14.0 passed. Host C imported only libSystem; host C++ also imported libc++. The iOS target graph includes `objc2-avf-audio` through `ios-media`, but no AVFAudio import was present. A minos-10.0 device link showed a strong `_VTIsHardwareDecodeSupported` import, so no pre-iOS-11 link/load claim is made. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, probes, or live VideoToolbox queries ran. The matrix remains 85/113 `B` (75.2%) and 28 `X`.

F25's static gate also asserts source/header/guide/plan output-pointer terms: valid aligned writable storage for the full synchronous call, caller protection from unsynchronized access, zero initialization before platform handling, nullness-only validation, and no pointer retention. This does not prove arbitrary C memory validity.

F26 `sh bindings/c/check-ios-camera-device-status.sh` and `sh bindings/c/check-ios-camera-device-status-link.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact AVFoundation/libSystem/libobjc device and Simulator imports, export parity, and probe minos 10.0/14.0 passed; Foundation was linked but dead-stripped. The runtime-guarded API floor remains iOS 4.0. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, probes, or camera queries ran. F26 adds C access to B62 without changing the capability matrix, which remains 85/113 `B` (75.2%) and 28 `X`.

The F26 static gate also asserts source/header/guide/plan/manifest output-pointer preconditions:
valid aligned writable memory for the full synchronous call, caller protection from unsynchronized
access, zero initialization before platform handling, nullness-only validation, and no pointer
retention. These assertions do not prove arbitrary C memory validity.

G108's locked offline workspace check, strict all-target/all-feature Clippy, and 47-package `no-std-check` passed on 2026-10-09. The host Clippy pass includes target-gated iOS-only link examples; no tests or example binaries ran. G109 F27 `sh bindings/c/check-ios-core-ml-status.sh` and `sh bindings/c/check-ios-core-ml-status-link.sh` passed. Host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact CoreML/Foundation/libSystem/libobjc device and Simulator imports, export parity, and minos 11.0/14.0 passed. Host C and C++ imported only libSystem; target C++ used `-nostdlib++`. The API floor is iOS 17.0. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, probes, model loads, inference, or live compute-device queries ran. F27 adds C access to B56 without changing the matrix, which remains 85/113 `B` (75.2%) and 28 `X`.

The F27 static/build gate also asserts source/header/guide/plan/manifest output-pointer preconditions:
valid aligned writable memory for the full synchronous call, caller protection from unsynchronized
access, zero initialization before platform handling, nullness-only validation, and no pointer
retention. These assertions do not prove arbitrary C memory validity.

G67 `platform/ios/callkit/ios-call-observer/check.sh` passed formatting, host/device/Simulator checks, strict Clippy, rustdoc, feature isolation, and CallKit/Foundation/libSystem/libobjc link/import audits. Probe minos is device 10.0 / Simulator 14.0; probes were not executed. See [G67](../PLAN_VALIDATION_IOS_CALL_OBSERVER.md).

G68 `platform/ios/ios-maps/check.sh` passed portable/host checks, strict Clippy, rustdoc, device/Simulator checks, feature isolation, and MapKit geometry link/import audits. The exact combined imports include ARKit, CoreLocation, Foundation, MapKit, libSystem, and libobjc; probe minos is 12.0/14.0 while the API floor is iOS 4.0. Probes were not executed. See [G68](../PLAN_VALIDATION_IOS_MAPKIT.md).

G69 `bindings/c/check-ios-call-observer.sh` passed opt-in feature isolation, locked host/device/Simulator checks, strict Clippy, Release archives, C11/C++17 links, symbol/import checks, and deployment metadata. Consumers and probes were not executed. See [G69](../PLAN_VALIDATION_C_ABI_CALL_OBSERVER.md).

G70 `bindings/c/check-ios-maps.sh` passed default/host feature isolation, generated binding-feature closure, locked host/device/Simulator checks and strict Clippy, Release archives, C11/C++17 links, symbol/import checks, and deployment metadata. C probes import MapKit and libSystem; C++ also imports libc++. Consumers and probes were not executed. See [G70](../PLAN_VALIDATION_C_ABI_MAPS.md).

G73 `platform/ios/ios-system-services/check.sh` passed host/device/Simulator checks, strict device/Simulator Clippy, iOS rustdoc, feature isolation, and ClassKit/Foundation/libSystem/libobjc Release link/import checks. Device minos is iOS 11.3 and Simulator minos is iOS 14.0; probes were not executed. See [G73](../PLAN_VALIDATION_IOS_CLASSKIT.md).

G75 `bindings/c/check-ios-classkit-deep-link.sh` passed opt-in feature isolation, locked host/device/Simulator checks, strict Clippy, Release archives, C11/C++17 links, export/import, selector and forbidden-data checks, and deployment metadata. The availability guard precedes pointer dereference; consumers and probes were not executed. See [G75](../PLAN_VALIDATION_C_ABI_CLASSKIT.md).

G76 and G77 are feasibility-only PushToTalk and CarPlay audits. No PTT support query/channel flow or CarPlay availability/entitled scene flow is implemented; rows 084 and 085 remain `X`.

G71, G72, and G74 are feasibility-only WeatherKit, RealityKit, and NetworkExtension audits. Their rows 091, 095, and 098 remain `X`; the reports define semantic, Swift ABI, REST authentication, host, entitlement, and provider lifecycle boundaries without claiming an implementation.

F28 `sh bindings/c/check-ios-speech-status.sh` and `sh bindings/c/check-ios-speech-status-link.sh`
passed host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact
Foundation/Speech/libSystem/libobjc imports, export parity, and minos 10.0/14.0. Host imports were
libSystem only; C++ target links used `-nostdlib++`. The gates are wired in macOS CI, with no
passing workflow run recorded. No tests, consumers, probes, permission prompts, or audio actions
ran. The matrix remains 85/113 `B` (75.2%) and 28 `X`.

The F28 static gate also asserts source/header/guide/plan output-pointer preconditions: valid aligned
writable `int64_t` storage for the full synchronous call, caller protection from unsynchronized
access, zero initialization before platform handling, nullness-only validation, and no pointer
retention. These assertions do not prove arbitrary C memory validity.

G112 F29 `sh bindings/c/check-ios-natural-language-status.sh` and
`sh bindings/c/check-ios-natural-language-status-link.sh` passed host/device/Simulator feature
isolation, strict Clippy, rustdoc, C11/C++17 links, exact Foundation/NaturalLanguage/libSystem/libobjc
imports, export parity, forbidden model-operation checks, and minos 17.0/17.0. Host imports were
libSystem only; target C++ links used `-nostdlib++`. The gates are wired in macOS CI, with no
passing workflow run recorded. No tests, consumers, probes, model loads, text input, asset requests,
or vector operations ran. The matrix remains 85/113 `B` (75.2%) and 28 `X`.

The updated F29 static gate also asserts source/header/guide/plan output-pointer preconditions:
valid, properly aligned writable `uint32_t` storage for the full synchronous call, caller protection
from unsynchronized access, zero initialization before platform handling, nullness-only validation,
and no pointer retention. These assertions do not prove arbitrary C memory validity.

G113 B6 `sh platform/ios/ios-sharing/check-clipboard-link-imports.sh` passed. It selects only the
`clipboard` feature, builds and inspects Release probes for device minos 10.0 and Simulator minos
14.0, checks exact UIKit/Foundation/libSystem/libobjc imports and clipboard selector markers, and
rejects share-only, Swift, and Python symbols. Probes were not executed; no live pasteboard action
or privacy prompt is claimed. The gate is wired in macOS CI, with no passing workflow run recorded.

The updated G113 gate also asserts `generalPasteboard`, `strings`, `firstObject`, and
`dataUsingEncoding:` alongside `hasStrings`, `setString:`, and `setItems:`; all selector markers passed
for device and Simulator.

G116 B7 `sh platform/ios/ios-sharing/check-share-link-imports.sh` passed. It selects only the
`share` feature, builds and inspects Release probes for device minos 10.0 and Simulator minos
14.0, checks exact CoreFoundation/Foundation/UIKit/libSystem/libobjc imports and share selector
markers, and rejects clipboard, Swift, and Python runtime symbols. Probes were not executed; no
live share UI or recipient behavior is claimed. The gate is wired in macOS CI, with no passing
workflow run recorded.

G114 F30 `sh bindings/c/check-ios-extension-support.sh` and
`sh bindings/c/check-ios-extension-support-link.sh` passed host/device/Simulator feature isolation,
strict Clippy, rustdoc, C11/C++17 links, exact imports, export parity, forbidden extension-loading
surface, and minos 12.0/14.0. Host C/C++ imports were only `libSystem.B.dylib`; device/Simulator
imports were exactly Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. The backend API floor
is iOS 4.0. Consumers and probes were inspected, not executed; no passing workflow run is recorded.

G115 F31 `sh bindings/c/check-ios-roomplan-status.sh` and
`sh bindings/c/check-ios-roomplan-status-link.sh` passed host/device/Simulator feature isolation,
Rust checks, strict Clippy, rustdoc, C11/C++17 header syntax and links, exact RoomPlan/libSystem
target imports, required RoomPlan symbols, and minos 16.0. Consumers and probes were not executed;
no RoomPlan call ran. Both gates are wired in macOS CI; no passing workflow run is recorded.

G111 `sh platform/ios/ios-files/check-app-data-link-imports.sh` builds Release consumers for
`ios-files` and `ios-preferences` on device and Simulator, checks exact per-crate imports and
minos 10.0/14.0, and is wired in macOS CI. The `ios-files` consumer imports CoreFoundation,
Foundation, `libSystem.B.dylib`, `libiconv.2.dylib`, and `libobjc.A.dylib`; the preferences consumer
imports Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. Probes were inspected, not executed;
no live app container or preference persistence behavior is claimed.

The updated G111 gate also asserts the B14 adoption export and both
`startAccessingSecurityScopedResource` / `stopAccessingSecurityScopedResource` selectors in the
`ios-files` artifact. These link assertions passed on device and Simulator; no URLSession provenance,
scope operation, or probe execution is claimed.

G117 F32 `sh bindings/c/check-ios-storekit2-status.sh` and
`sh bindings/c/check-ios-storekit2-status-link.sh` passed after root integration. Host C/C++ imports
only `libSystem.B.dylib`; device/Simulator imports weak StoreKit plus `libSystem.B.dylib`, with the
getter as a weak symbol and minos 10.0/14.0. The API/symbol floor is iOS 15.0; no runtime fallback
check below that floor, tests, consumers, or probes ran. The gates are wired in macOS CI; no passing
workflow run is recorded.

G118 F33 `sh bindings/c/check-ios-game-status.sh` and
`sh bindings/c/check-ios-game-status-link.sh` passed after root integration. Host C/C++ imports only
`libSystem.B.dylib`; device/Simulator imports Foundation, GameKit, `libSystem.B.dylib`, and
`libobjc.A.dylib`, with minos 10.0/14.0. The SDK API floor is iOS 4.1; the signed app needs
`com.apple.developer.game-center`. No tests, auth flow, live player query, consumers, or probes ran.
The gates are wired in macOS CI; no passing workflow run is recorded.

G103 B4 `sh platform/ios/ios-notifications/check-link-imports.sh` passed after the
`pending_request_count()` future was added. Its device/Simulator Release gate checks exact
Foundation/UserNotifications/runtime imports, the undefined-symbol denylist, and that the
compile-only example selects the new API. No tests, callback, live count, permission prompt, or probe
execution occurred; the gate is wired in macOS CI with no passing workflow run recorded.

G13's device/Simulator `ios-data` check and strict Clippy commands plus its focused import-script
invocation are wired in `.github/workflows/ci.yml` and passed locally. No CI workflow run or
runtime `CFData`/`NSData` behavior is claimed.

G14's device/Simulator `ios-url` check and strict Clippy commands plus its corrected focused
import-script invocation are wired in `.github/workflows/ci.yml` and passed locally. No CI workflow
run or runtime URL/parser behavior is claimed.

G15's `ios-ui` device/Simulator check and strict Clippy commands plus its CoreGraphics/layout
probe invocation are wired in `.github/workflows/ci.yml` and passed locally. The exact import and
layout results are recorded above; the probes build but are not executed, and no CI workflow run
is recorded. Compile/link/layout evidence does not establish runtime geometry parity or visual
behavior.

G16's `ios-media` device/Simulator check and strict Clippy commands plus its CoreMedia/layout
probe invocation are wired in `.github/workflows/ci.yml`. The target checks and Clippy passed
locally, as did the final link/import script after it added `-Wl,-dead_strip_dylibs` and the exact
CoreMedia plus `libSystem.B.dylib` allowlist. Probes are build-only, and no CI workflow run or
runtime media behavior is claimed.

G17 reuses the `ios-ui` device/Simulator check and strict Clippy CI gates and adds
`check-text-metrics-link-imports.sh`. The local target checks and script passed on Xcode 26.6 / SDK
26.5; Release probes import only CoreText, CoreFoundation, and `libSystem.B.dylib`, and the C
fixture validates public CoreText function signatures plus LP64 layouts. Hosted run `38075483431`
passed G17's wired device/Simulator, Clippy, and text-metrics link/signature gates on Xcode 27.0 /
SDK 27.0. The probe binaries were not executed, and no live font output or `UILabel` parity is
claimed.

G18's 11 portable `framework-connection` contract tests, no-default-features check, locked device/Simulator `ios-connection` checks, strict all-target Clippy, and `sh platform/ios/ios-connection/check-link-imports.sh` passed locally on Rust 1.94.1 / Xcode 26.6 / SDK 26.5. The arm64 probes import exactly `Network` and `libSystem.B.dylib`, with minimum iOS 12.0 / Simulator 14.0. Hosted run `38075483431` passed the wired Xcode 27 device/Simulator, Clippy, and connection link/import gates. These compile/link/import checks do not establish live DNS, peer identity, TLS outcomes, network reachability, or throughput; the linked probes are not executed.

G19's Rust 1.94.1 `framework-background` no-default check/Clippy plus device/Simulator `ios-background-tasks` checks, strict Clippy, and `sh platform/ios/ios-background-tasks/check-link-imports.sh` passed locally on Xcode 26.6 / SDK 26.5. The probes import `BackgroundTasks`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; their minos is device 13.0 / Simulator 14.0. Probes are not executed, and no task launch, expiry, scheduler timing, or relaunch behavior is claimed. Hosted run `38075483431` also passed G19's portable, device/Simulator, Clippy, and import-audit gates on Xcode 27.0 / SDK 27.0.

G20's portable image metadata checks and `sh platform/ios/ios-image-io/check-ios.sh` passed locally on Rust 1.94.1 / Xcode 26.6 / SDK 26.5 and were requalified on the current tree at the same toolchain. Device and Simulator builds passed, strict Clippy passed, and the ImageIO/CoreFoundation import and feature audit passed; probes were not executed and no live image was used. G21's portable Photos tests/check/Clippy/rustdoc and iOS Photos device/Simulator checks, strict Clippy, API-floor fixture, and `sh platform/ios/ios-photos/check-link-imports.sh` passed in an isolated workspace. The probes import exactly Photos, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib` with minos 14.0; they were not executed and no live prompt was used. G22's portable no-default check/Clippy/rustdoc, iOS device/Simulator checks and strict Clippy, and UIKit selector/import audit passed locally; no probe was executed and no live expiry callback was observed. G23's Contacts portable tests/no-default check, target checks/strict Clippy, docs/format, and generated-feature review passed in an isolated workspace; no live prompt, contacts data access, or import audit was run. G24's Calendar portable/host tests, no-default check, warning-denied rustdoc, device/Simulator checks and strict Clippy, docs check, and EventKit import probes passed in an isolated workspace; no live prompt or Calendar data access was exercised. G25's HealthKit portable tests/no-default check/strict Clippy/rustdoc and device/Simulator target checks/strict Clippy passed in the isolated worktree and root checkout; no app link, entitlement validation, or live permission flow was run. G26's Bluetooth authorization portable tests/no-default check and target checks/strict Clippy passed; no manager or operation was tested. G27's portable WebKit contract and target/surface gates passed in an isolated worktree, with no page or network action. G28's Bluetooth discovery portable and target gates passed, with no live scan. G29's iCloud identity portable and target gates passed in a validation copy, with no live account query. Hosted run `38075483431` on source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b` passed the G20–G29 package and import gates wired into the Xcode 27 lane. These compile/lint and link/import gates do not establish runtime behavior, and no live image, prompt, contact data access, network action, Bluetooth scan, or account query is claimed.


G30 `ios-media-authorization/check.sh` runs portable status contract tests, iOS device/Simulator compile and strict Clippy gates, rustdoc, and the status-only surface guard; it passed in the integrated checkout. G31 portable NFC tests, no-default check, strict Clippy, and iOS device/Simulator compile and strict Clippy gates passed in the integrated checkout. G32 records the HomeKit audit: no prompt-free status query was implemented because first `HMHomeManager` use can prompt. G33 `ios-nearby/check.sh` gates the iOS 16+ precise-distance capability query on device and Simulator; it passed in the integrated checkout. G34 `ios-auth/check.sh` gates the ATT portable contract and iOS status-only adapter; it and `ios-auth/check-link-imports.sh` passed in the integrated checkout. G35 `ios-metal/check.sh` gates the default Metal-device presence contract; portable tests/no-default check/strict Clippy/rustdoc, iOS device/Simulator checks/strict Clippy, and exact Release imports/symbols passed in the integrated checkout. Probes were not executed and no GPU work was submitted. The full workspace check, strict Clippy, tests/doctests, `no-std-check`, and host/device/Simulator `no-std-link-probe` passed after D31–D35 integration. G36 `ios-device-integrity/check.sh`, G37 `ios-watch-connectivity/scripts/check.sh`, G38 `ios-accessory/scripts/check.sh`, G39 `ios-replaykit/check.sh`, and G40 `ios-sound-analysis/check.sh` passed locally on Xcode 26.6 / SDK 26.5. These six package gates also passed hosted steps 172–177 on macOS 15 and Xcode 27 in run `38075483431`; the macOS-only steps were skipped on Ubuntu. The probes were linked, not run. These static gates do not exercise live permission prompts, capture/recording, NFC sessions or tags, Nearby Interaction sessions/ranging, ATT status reads/prompts/tracking, Metal device behavior, HomeKit behavior, support queries, accessory communication, microphone access, or audio analysis.

macOS CI also runs `sh bindings/c/check-ios-transfer.sh`, `sh bindings/c/check-ios-clipboard.sh`, and `sh bindings/c/check-ios-share.sh`, which check the opt-in transfer, clipboard, and share C ABIs without a live download, pasteboard access, share UI, or UIKit event. The transfer script links a C probe for each iOS target; `otool -L` requires exact direct imports `CoreFoundation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`, while C++ fixture links also allow `libc++.1.dylib`. `nm -u` scans each C/C++ probe, not the static archives, for Swift/Python runtime and unrelated capability symbols. The script does not execute probes; this evidence covers those probe links only and does not prove a live download or URLSession event

The C7 App Intents Stage 0 build audit is a manual macOS check, not a CI gate. It uses temporary Swift source and build products outside the checkout, observes the normal Xcode metadata pipeline, and does not test Stage 1 intent discovery, invocation, installation, or runtime behavior.

`.github/workflows/ci.yml` defines Rust checks on `macos-15` and `ubuntu-24.04`. Both jobs run formatting, workspace Clippy and tests, portable `no_std` checks, dependency and ABI inventories, rustdoc, and docs/zero-Swift-source checks. The macOS job also installs the iOS device and Simulator targets; runs the configured C/C++ ABI checks, iOS package checks and link/import gates, plus locked device/Simulator checks and strict Clippy; checks/tests/lints `framework-nfc`; builds the minimal Release Simulator app, audits its imports and `Info.plist`, and records the Xcode/SDK environment. The workflow is the authoritative list of package and script commands.
