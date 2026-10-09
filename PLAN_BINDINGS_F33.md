# PLAN_BINDINGS_F33.md — F33: Game Center local-player status C ABI

## Objective

Expose only B55's synchronous local-player authentication snapshot through one opt-in C function. Add no
authentication flow, callback, native handle, identity, profile, game data, or portable API expansion

## Candidate and bounds

B55 already exposes one owned `LocalPlayerAuthenticationStatus` from `GKLocalPlayer.localPlayer` and
`GKLocalPlayer.isAuthenticated`. The retained native player stays within the backend call and drops before
return. B55 does not set `authenticateHandler`, run authentication initialization, present UI, install a
status observer, or expose player data. The typed `objc2-game-kit` 0.3.2 binding provides both selected API
calls. The SDK declaration floor is iOS 4.1

`Authenticated` writes one; `NotAuthenticated` writes zero. The portable enum is non-exhaustive, so any future
unrecognized case returns `FRAMEWORK_STATUS_UNAVAILABLE` with zero output; this avoids mapping an unknown
case to false. A false current result can include an offline temporary player or an uninitialized Game Center
service and does not prove that no account exists

The configured app still needs its signed `com.apple.developer.game-center` entitlement. This query does not
validate the entitlement, sign-in, account setup, or service initialization. No privacy usage-description key
is required for the read

Other remaining candidates are outside F33: B53 SafetyKit has an unresolved getter-specific entitlement
prerequisite; B60 is deprecated StoreKit 1 purchase ability; B79 VPN status requires a preference-load
operation and native result semantics. They do not need to change for the one-call B55 contract

## Exact C contract

- Cargo feature: `ios-game-status`; default remains empty
- Header: `bindings/c/include/framework_ios_game_status.h`
- Export: `FrameworkStatus framework_ios_game_status_is_local_player_authenticated(uint8_t *out_authenticated)`
- On iOS, return `FRAMEWORK_STATUS_OK` with exactly zero or one from B55's owned enum
- A future unrecognized `LocalPlayerAuthenticationStatus` returns `FRAMEWORK_STATUS_UNAVAILABLE` and zero
- A valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` and zero
- Null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A caught panic returns
  `FRAMEWORK_STATUS_PANIC` and zero output
- Output is one caller-owned byte, initialized to zero before platform handling. The API checks nullness only.
  A non-null pointer must be valid, aligned, and writable for the full synchronous call; the caller must
  prevent unsynchronized access. The pointer is not retained
- The call runs on the caller's thread. No main-thread, queue, or thread-safety guarantee is added
- The native player is retained only for B55's synchronous read and dropped before return. No GameKit object,
  player data, callback, or native pointer crosses C

## Availability and link bounds

The SDK-declared API floor is iOS 4.1. F33's C/C++ Release probes use device minos 10.0 and Simulator minos
14.0 as link settings; these do not change the API floor. Device and Simulator direct imports must be measured
from the linked C/C++ consumers. The API requires GameKit and Foundation; F33 must not import a Swift runtime
or unrelated capability framework

## Files and root wiring

F33 owns `bindings/c/src/ios_game_status.rs`, `bindings/c/include/framework_ios_game_status.h`,
`bindings/c/check-ios-game-status.sh`, `bindings/c/check-ios-game-status-link.sh`,
`docs/bindings/ios-game-status.md`, and this plan. The isolated implementation also adds the optional
`framework-game`/`ios-game` dependencies and feature, module/re-export, and ABI manifest entry to `bindings/c`.
Root integration is complete: Cargo.lock, both macOS CI gates, aggregate plan evidence, and the docs index
are wired. Capability rows and support counts did not change

## Evidence and limits

The source/API facts and entitlement/runtime limits are recorded in `PLAN_IOS_GAMEKIT_STATUS.md` and
`docs/ios/game-center-status.md`. F33's isolated validation passed with Rust 1.94.1, Xcode 26.6 build
17F113, and iPhoneOS/iPhoneSimulator SDK 26.5. This toolchain is below the repository's Xcode 27.x baseline

- `cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features ios-game-status`
  passed and refreshed only the isolated snapshot's Cargo.lock
- `sh bindings/c/check-ios-game-status.sh` passed formatting, shell syntax, JSON and source/header contract,
  host/default/iOS feature-tree isolation, and standalone C11/C++17 header syntax
- `sh bindings/c/check-ios-game-status-link.sh` passed locked host/device/Simulator checks, strict Clippy,
  host rustdoc, Release archive builds, C11/C++17 host/device/Simulator links, export parity, imports, and
  minos inspection
- Host C/C++ imported only `libSystem.B.dylib`. Device and Simulator C/C++ imported Foundation, GameKit,
  `libSystem.B.dylib`, and `libobjc.A.dylib`; no Swift runtime or unrelated capability framework imported.
  Device minos was 10.0 and Simulator minos was 14.0; the SDK API floor remains iOS 4.1
- Linked consumers were inspected but not executed. No tests, auth handler, sign-in prompt, live player read,
  signed entitlement check, or Game Center UI ran. No passing CI workflow run is recorded
- Both gates passed in the integrated checkout after root wiring; linked consumers were inspected but not
  executed

F33 reports one momentary local-player value only. It does not establish account existence, service readiness,
entitlement validity, future state, identity, parity, or performance
