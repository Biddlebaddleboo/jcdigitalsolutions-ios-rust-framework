# PLAN_IOS_VPN_STATUS.md — B79: Personal VPN status snapshot

## Implementation status

- Portable values, the iOS future adapter, package-local gates, and scoped guides are present
- Root refreshed `Cargo.lock`; `objc2-network-extension` 0.3.2 and the two new workspace packages resolve under `--locked`
- `sh platform/ios/ios-vpn/check.sh` passes compile, strict Clippy, rustdoc, feature-tree, and link/import checks; device and Simulator probes were inspected but not executed, and no VPN query or tests ran
- The link gate uses device minos 10.0 and Simulator minos 14.0; rustc 1.94.1 warns below iOS 10.0 and emits minos 10.0 when asked for 8.0
- B79 code and focused gates are complete; root integrated row 098 as a scoped `B` partial, added the macOS CI gate, and indexed the capability and iOS guides

## Objective

Add one read-only, asynchronous Rust query for the calling app's Personal VPN profile status through iOS `NEVPNManager`. Preserve the native status and preference-load error as distinct owned Rust values. This is a partial implementation of row 098, not a general NetworkExtension/VPN facade

## Scope boundary

- Read only the calling process's `NEVPNManager.sharedManager()` profile
- Call `loadFromPreferencesWithCompletionHandler:` before reading `connection.status`
- Preserve the six public status values: Invalid, Disconnected, Connecting, Connected, Reasserting, and Disconnecting
- Preserve a load failure as its native error domain and code; do not map failure to Disconnected or Invalid
- Use a caller-owned Rust future; the native request begins when the request function runs, and dropping the future abandons Rust interest without claiming native cancellation
- Keep Apple's callback context explicit: the preference-load callback runs on the caller's main thread
- Require and document the Personal VPN entitlement `com.apple.developer.networking.vpn.api = ["allow-vpn"]`; a successful compile or link does not establish entitlement approval or runtime access
- For the B79 status request, do not read, save, or remove configuration properties. B200 adds a separate query for only the two enabled flags; neither request saves or removes a profile or starts/stops a tunnel, packet/flow tunnel, DNS, proxy, filter, or provider extension
- Do not claim global device VPN state, route coverage, reachability, tunnel health, or system-wide configuration
- Add no Swift source, Swift ABI, C ABI, UI, permission prompt, or global service registry

## Planned packages

- `crates/framework-vpn`: `#![no_std]` portable status and owned native load/query-error contract
- `platform/ios/ios-vpn`: iOS 8.0 `NEVPNManager` adapter using generated `objc2-network-extension` 0.3.2 bindings with defaults off and only its `block2` feature enabled; the generated crate has no per-class `NEVPNManager` or `NEVPNConnection` feature
- The iOS package's direct binding features are `block2/alloc`, `objc2/std`, `objc2-foundation/{NSError,NSString}`, and `objc2-network-extension/block2`; no generated NetworkExtension Objective-C ABI is handwritten
- The queried Apple API floor is iOS 8.0, but rustc 1.94.1 reports iOS 10.0 as its minimum supported deployment target; this workstream does not claim a linked Rust app deployment target earlier than iOS 10.0
- `docs/capabilities/vpn.md` and `docs/ios/vpn-status.md`: partial-scope, entitlement, lifecycle, and native-escape-hatch notes
- `PLAN_IOS_VPN_STATUS.md`: B79 implementation and validation record

The workspace already uses `crates/*` and `platform/ios/*` member globs. Each crate must live in an immediate member directory and must not add a nested `[workspace]`

## Public contract

`PersonalVpnStatus` is a non-exhaustive portable value matching Apple's six public `NEVPNStatus` cases and preserving a future raw value as `Unknown(i64)`. The portable contract uses fixed-width `i64`, not pointer-width `isize`; the iOS adapter losslessly widens the native `NSInteger` on supported 32-bit and 64-bit Darwin targets. `Invalid` is preserved as the platform value; it is not a global "VPN off" claim. `PersonalVpnLoadError` preserves an owned domain string and native `NSInteger` code widened to fixed-width `i64`. `PersonalVpnQueryError` keeps that load error distinct from a callback-thread mismatch or a caught Rust panic. The iOS future returns `Result<PersonalVpnStatus, PersonalVpnQueryError>`

The request function requires `ios_runtime::main_thread::MainThread`, starts the native request as the function runs, and returns a future that retains the proof plus a `PhantomData<Rc<()>>` marker so it is `!Send` and `!Sync`. The callback uses an `Arc<Mutex<...>>` containing only owned Rust values and a waker; it verifies `MainThreadMarker::new()` before querying NetworkExtension. Dropping the future detaches its Rust result and waker but does not cancel the preference load

The iOS API floor is 8.0. The generated binding methods are `unsafe`; every call must have a local safety note that identifies the Apple API, retained manager/connection lifetime, and block lifetime. The escaping block must own all Rust state it touches. The result must be copied before the callback's `NSError` pointer expires. No Apple object or borrowed pointer may escape the callback

## Evidence and sources

The local SDK is Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5. `NEVPNManager`, `sharedManager`, `loadFromPreferencesWithCompletionHandler:`, `connection`, `NEVPNConnection.status`, and `NEVPNStatus` have an iOS 8.0 floor. `NEVPNManager` and `NEVPNConnection` are documented thread-safe; Apple's preference-load callback runs on the caller's main thread. A missing/disabled configuration may report Invalid; that must remain distinct from other status values

Primary references: [Apple `NEVPNManager`](https://developer.apple.com/documentation/networkextension/nevpnmanager), [load preferences callback](https://developer.apple.com/documentation/networkextension/nevpnmanager/loadfrompreferences%28completionhandler%3A%29?language=objc), [connection](https://developer.apple.com/documentation/networkextension/nevpnmanager/connection), [status](https://developer.apple.com/documentation/networkextension/nevpnconnection/status), [Personal VPN entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.vpn.api), [`objc2-network-extension` 0.3.2 `NEVPNManager`](https://docs.rs/objc2-network-extension/0.3.2/objc2_network_extension/struct.NEVPNManager.html), and [`NEVPNStatus`](https://docs.rs/objc2-network-extension/0.3.2/objc2_network_extension/struct.NEVPNStatus.html)

## Validation gates

- `cargo check --locked -p framework-vpn --no-default-features`
- `cargo clippy --locked --lib -p framework-vpn --no-default-features -- -D warnings`
- `cargo check --locked -p ios-vpn --target aarch64-apple-ios`
- `cargo check --locked -p ios-vpn --target aarch64-apple-ios-sim`
- `cargo clippy --locked --lib -p ios-vpn --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked --lib -p ios-vpn --target aarch64-apple-ios-sim -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps -p framework-vpn -p ios-vpn --target aarch64-apple-ios`
- `cargo tree --locked -p ios-vpn --target aarch64-apple-ios -e features` shows `objc2-network-extension/block2` without its default features
- `sh platform/ios/ios-vpn/check-link-imports.sh` builds and inspects device and Simulator probes; it never executes them
- Device probe minos is 10.0 because rustc 1.94.1 warns below that minimum and emits minos 10.0 even if `IPHONEOS_DEPLOYMENT_TARGET=8.0` is set. Simulator minos is 14.0 as a probe choice. Neither probe minos changes the iOS 8.0 API floor
- Both link probes import only NetworkExtension, Foundation, libSystem, and libobjc; the device probe contains the `NEVPNManager` class name and the `sharedManager`, `loadFromPreferencesWithCompletionHandler:`, `connection`, and `status` selectors. `NEVPNConnection` has no emitted class-name string because this code obtains it via the `connection` getter
- Package-scoped source/feature, formatting, whitespace, rustdoc, and `git diff --check` gates run from `platform/ios/ios-vpn/check.sh`
- Root integration records row 098 as a portable-contract/iOS-backend partial, including the Personal VPN entitlement, iOS 8.0 API floor, rustc 1.94.1 device minos 10.0, arm64 Simulator minos 14.0, and exact NetworkExtension/Foundation/libSystem/libobjc imports

Do not add or run tests in this workstream. No entitlement, user authorization, preference contents, live VPN status, tunnel, network route, or device behavior is exercised by compile or link gates

## Completion criteria

- Portable status/error types compile and pass strict Clippy on the host; the iOS future compiles and passes strict Clippy for device and Simulator targets
- The backend reads status only after successful preference load and maps all six public values while preserving unknown raw values separately from load errors
- Future drop detaches Rust interest without retaining caller state or claiming cancellation; callback and waker state contains only owned `Send + Sync` values and remains safe for the documented main-thread callback
- [x] The manifest row becomes partial (`B`) only after the implementation and focused gates pass, with the Personal VPN entitlement and all broad NetworkExtension limits stated
- [x] CI and docs indexes invoke and describe the focused gates; no generic NetworkExtension claim is added

## B200 — Personal VPN configuration flags

B200 adds a second narrow read-only query to the existing row 098 partial. `ios-vpn::request_personal_vpn_configuration(MainThread)` asynchronously loads the calling app's preferences and, only after a successful load, reads `NEVPNManager.isEnabled` and `NEVPNManager.isOnDemandEnabled`. It returns `PersonalVpnConfigurationFlags { enabled, on_demand_enabled }` through `IosPersonalVpnConfigurationFuture`; the existing status API and B79 behavior remain intact. Preference-load failures, callback-thread mismatch, and caught Rust panics use the existing `PersonalVpnQueryError` contract.

The two Booleans report only the loaded configuration properties. Apple documents that only one Personal VPN configuration can be enabled at once; if another is enabled, `isEnabled` is set false in preferences and a reload is needed to observe a change. `isOnDemandEnabled` reports the Connect On Demand capability flag only; B200 does not read the rule list or claim automatic connection behavior. Neither accessor is called as a setter. The request keeps the `allow-vpn` entitlement, caller-main-thread, future-drop, and iOS API-floor constraints from B79.

## B448 — fixed-width native VPN status/error values

B448 corrects the portable API's pointer-width `isize` fields. `PersonalVpnStatus::Unknown`, `from_native_raw`, `native_raw`, and `PersonalVpnLoadError::code` now use `i64`. The iOS adapter widens `NEVPNStatus` and `NSError.code()` from native `NSInteger`; supported Darwin `NSInteger` widths (32 and 64 bits) fit without loss. The status semantics, error-domain ownership, iOS availability, future behavior, and capability-row coverage do not change. This aligns portable values with the fixed-width semantic-data invariant and avoids a pointer-width-dependent public contract.

Focused non-test validation: locked/offline host, iOS device, and iOS Simulator checks; strict Clippy for the portable and iOS packages; iOS rustdoc with warnings denied; package format, docs-check, zero-Swift-source, and scoped diff gates. No tests, preference-load calls, runtime VPN queries, app launches, probes, or entitlement checks were run.

### B200 declaration and binding evidence

The installed Xcode 26.6 build `17F113` iPhoneOS 26.5 SDK declares `NEVPNManager.onDemandEnabled` with getter `isOnDemandEnabled` and `NEVPNManager.enabled` with getter `isEnabled`, both `API_AVAILABLE(ios(8.0))`, in `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/NetworkExtension.framework/Headers/NEVPNManager.h` (header lines 115–118 and 145–148). The locally installed `objc2-network-extension` 0.3.2 binding declares `NEVPNManager::isOnDemandEnabled(&self) -> bool` and `NEVPNManager::isEnabled(&self) -> bool` in `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-network-extension-0.3.2/src/generated/mod.rs` (the `NEVPNManager` impl around lines 738 and 790). Both generated calls are `unsafe`; implementation safety notes tie them to a retained manager and the successful preference callback.

Apple documents [`isEnabled`](https://developer.apple.com/documentation/networkextension/nevpnmanager/isenabled) as the enabled state of the VPN configuration and notes the other-profile false case and reload requirement. Apple documents [`isOnDemandEnabled`](https://developer.apple.com/documentation/networkextension/nevpnmanager/isondemandenabled) as the Connect On Demand capability toggle, defaulting to false. The configuration operation adds no Swift, Swift ABI, handwritten Objective-C ABI, manager lifecycle, rule read, user prompt, or device-state query.

Local SDK inspection and static gates use Xcode 26.6 / iPhoneOS SDK 26.5, which is below the repository's Xcode 27.x baseline. These results do not establish the Xcode 27.x gate result. The public API floor remains iOS 8.0, while rustc 1.94.1's minimum supported deployment target remains iOS 10.0 for device builds; the existing Simulator probe target remains iOS 14.0.

### B200 focused static gates

- Passed `cargo +1.94.1 check --locked -p framework-vpn --no-default-features` and strict portable Clippy
- Passed `cargo +1.94.1 check --locked -p ios-vpn --target aarch64-apple-ios` and `cargo +1.94.1 check --locked -p ios-vpn --target aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 clippy --locked --lib -p ios-vpn --target aarch64-apple-ios -- -D warnings` and the equivalent `aarch64-apple-ios-sim` command
- Passed `RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p framework-vpn -p ios-vpn --target aarch64-apple-ios`, both package formatting checks, shell syntax, static source-scope checks, `cargo +1.94.1 tree --locked -p ios-vpn --target aarch64-apple-ios -e features`, `cargo +1.94.1 xtask docs-check`, and scoped `git diff --check`
- The package gate `sh platform/ios/ios-vpn/check.sh` also builds and inspects linked device/Simulator probes; its link/import portion was not run for B200 because this audit explicitly excludes probes
- The source-scope gate requires both typed getter calls and rejects `setEnabled`, `setOnDemandEnabled`, `onDemandRules`, preference writes/removal, tunnel control, provider APIs, and Swift source
- The link/import gate requires the emitted `isEnabled` and `isOnDemandEnabled` selectors and the existing NetworkExtension/Foundation/libSystem/libobjc imports
- No tests, app runs, probe builds/executions, live manager calls, entitlement checks, or device queries were run for B200
