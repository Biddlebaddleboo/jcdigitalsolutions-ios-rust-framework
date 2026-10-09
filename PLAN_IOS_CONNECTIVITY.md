# PLAN_IOS_CONNECTIVITY.md — Workstream B18: iOS Network Path Snapshot

## Status

Implementation and compile/link gates complete. The inspected Xcode 26.6 (build 17F113) / iOS SDK
26.5 headers expose the public path-monitor C API from iOS 12.0. This remains an informational
snapshot; it must not gate or preflight a URLSession or other network operation. The host is below
the planned Xcode 27.x baseline, and no live path-change or cancellation runtime evidence exists

## Objective

Implement D15's one-shot local path snapshot with public Network.framework C APIs, without Swift
source/runtime, private symbols, or changes to B3 foreground HTTP

## Dependencies

- Foundation A and `ios-runtime` are integrated
- D15 `framework-connectivity` is integrated; see `PLAN_CAPABILITIES_CONNECTIVITY.md`
- B3 `ios-network` remains the owner of HTTP request/response and URLSession transport

## Write scope

- `PLAN_IOS_CONNECTIVITY.md`
- `platform/ios/ios-connectivity/**`
- `docs/ios/connectivity.md`
- a focused `check-link-imports.sh` for this crate
- workspace dependency entry only if required; root owns version and lockfile reconciliation

Do not edit `framework-network`, D1/B3, root `Cargo.toml`, `Cargo.lock`, shared `ios-runtime`,
capability status manifest, `PLAN_CAPABILITIES.md`, CI, or unrelated backend crates. Root owns
workspace, lockfile, CI, and matrix integration. Add no Swift source and do not use
`SCNetworkReachability`

## Backend requirements

- Add an iOS-only `ios-connectivity` crate that implements D15's static `NetworkPathBackend`
- Use the public C path-monitor API declared by the installed `<Network/Network.h>` SDK headers:
  create a monitor, set a delivery queue, update handler, and cancel handler, then start it; read
  the first non-null path update's native status and cancel the monitor
- Use generated/header-checked bindings or a small C shim compiled against the public SDK headers.
  Do not guess or hand-maintain undocumented Apple ABI declarations. Any build dependency used
  for the shim must be justified and kept behind this platform crate. The implementation uses a
  local C shim and the crate-only `cc` build dependency to compile it with `-fblocks` against
  `<Network/Network.h>`
- Preserve D15's four status values exactly. Map native invalid status to `Unknown`; do not treat
  `Satisfied` as Internet or endpoint reachability
- Start the monitor no earlier than the returned future's first poll. Resolve from the first
  non-null path update and then cancel. Map `nw_path_status_invalid` to terminal `Unknown`; do
  not wait for a later status. A future that is not polled starts no native work
- On a pending future drop, detach/invalidate the incomplete callback cell before cancellation. A
  completion claim that wins the cell mutex before detach is terminal and may call its already-
  claimed Waker after future drop; updates that reach the cell after detach are ignored. Keep the
  callback context until `nw_path_monitor_set_cancel_handler` fires on the configured queue;
  this is the cancellation-completion boundary after which the update handler will not fire again.
  Ignore duplicate/late updates and release the context exactly once
- Deliver no native callback panic across the C/block boundary. No user callback is exposed by
  this one-shot API
- Do not request permission, create a connection, perform DNS, access a host, or gate/retry a
  request based on the path result
- Keep the iOS 12.0 base API floor; use no iOS 13/14 path properties in this slice
- Document the callback queue, status mapping, lifecycle/cancellation, path-vs-endpoint boundary,
  and that no live request or path-change behavior is verified

## Link and validation

- Check and strict-Clippy the crate for `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Link a minimal probe for device and simulator and inspect imports. Require Network.framework and
  only its actual required system runtime imports; reject Swift runtime/source and unrelated
  capability imports. Set the probe's device deployment target to iOS 12.0 and its simulator target
  to iOS 14.0, then inspect the result with `vtool`
- Run formatting, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and
  `git diff --check`
- Do not claim path observation, connectivity, request success, or cancellation runtime behavior
  from compile/link evidence. No physical-device or simulator path-change test is required in this
  slice
- Report exact SDK header declarations, generated-binding or C-shim method, imports, API floor,
  checks, deviations, and remaining runtime limits

## Validation record

The C shim compiles against `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/Network.framework/Headers/path_monitor.h` and `path.h`. `path_monitor.h` marks `nw_path_monitor_create`, `nw_path_monitor_set_queue`, `nw_path_monitor_set_cancel_handler`, `nw_path_monitor_set_update_handler`, `nw_path_monitor_start`, and `nw_path_monitor_cancel` as `ios(12.0)`. `path.h` marks `nw_path_get_status` as `ios(12.0)` and defines the four native status constants used by the C shim. The shim maps them to private callback tags `FRAMEWORK_PATH_STATUS_UNKNOWN = 0`, `FRAMEWORK_PATH_STATUS_SATISFIED = 1`, `FRAMEWORK_PATH_STATUS_UNSATISFIED = 2`, and `FRAMEWORK_PATH_STATUS_SATISFIABLE = 3`; these are shim values, not Apple enum values. Rust maps every other tag to `Unknown`.

On Xcode 26.6 (build 17F113), SDK 26.5, and Rust 1.94.1, these commands passed:

- `xcrun --sdk iphoneos clang -target arm64-apple-ios12.0 -fblocks -std=c11 -Wall -Wextra -Werror -fsyntax-only -I platform/ios/ios-connectivity/src platform/ios/ios-connectivity/src/network_shim.c`
- `xcrun --sdk iphonesimulator clang -target arm64-apple-ios14.0-simulator -fblocks -std=c11 -Wall -Wextra -Werror -fsyntax-only -I platform/ios/ios-connectivity/src platform/ios/ios-connectivity/src/network_shim.c`
- `cargo +1.94.1 check --locked --offline -p ios-connectivity`
- `cargo +1.94.1 check --locked --offline -p ios-connectivity --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-connectivity --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-connectivity --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-connectivity --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p ios-connectivity --no-deps`
- `sh platform/ios/ios-connectivity/check-link-imports.sh`
- `cargo +1.94.1 --locked --offline xtask docs-check`
- `cargo +1.94.1 --locked --offline xtask zero-swift-source`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-connectivity/Cargo.toml -- --check`
- `sh -n platform/ios/ios-connectivity/check-link-imports.sh`
- `git diff --check`

The link-only device probe imports `Network` and `libSystem.B.dylib`; the simulator probe has the same direct imports. `nm -u` found the expected `nw_path_monitor_*`, `nw_path_get_status`, `nw_release`, Dispatch, block-runtime, and Rust/libSystem symbols; the script rejected Swift/Python runtime, Objective-C class, and unrelated Network capability symbols. `vtool -show-build` reports device minos 12.0 / SDK 26.5 and simulator minos 14.0 / SDK 26.5. Neither probe was executed. No tests, live requests, device/simulator path updates, cancellation timing, runtime parity, or performance were verified.
