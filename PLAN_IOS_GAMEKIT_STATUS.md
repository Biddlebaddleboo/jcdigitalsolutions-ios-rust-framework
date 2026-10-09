# PLAN_IOS_GAMEKIT_STATUS.md — Workstream B55: Non-Prompting Game Center Status

## Objective

Implement D50's point-in-time local-player authentication status read with `GKLocalPlayer.localPlayer` and `GKLocalPlayer.isAuthenticated`. Do not initialize/authenticate the local player or expose player data.

## API and side-effect audit

The installed Xcode 26.6 / iPhoneOS 26.5 SDK declares `GKLocalPlayer` available from iOS 4.1; its `localPlayer` and inherited `isAuthenticated` accessors have no later availability annotation. `localPlayer` is the accessor used for a shared instance. Apple documents Game Center authentication initialization as setting `authenticateHandler`; that handler may receive a view controller and Game Center may show a brief initialization screen. B55 only reads `localPlayer` and `isAuthenticated`, does not set the handler, and does not perform the documented authentication initialization. The `localPlayer` header notes an offline temporary player can exist if no account is set up.

`objc2-game-kit` 0.3.2 exposes `GKLocalPlayer::localPlayer() -> Retained<GKLocalPlayer>` and `GKLocalPlayer::isAuthenticated(&self) -> bool` with `GKBasePlayer`, `GKPlayer`, and `GKLocalPlayer` features. Both generated methods are `unsafe`; the implementation limits the unsafe calls to the live retained local player and scalar read.

Primary Apple references:

- [`GKLocalPlayer`](https://developer.apple.com/documentation/gamekit/gklocalplayer)
- [`isAuthenticated`](https://developer.apple.com/documentation/gamekit/gklocalplayer/isauthenticated?language=objc)
- [`authenticateHandler`](https://developer.apple.com/documentation/gamekit/gklocalplayer/authenticatehandler?language=objc)
- [Authenticating a player](https://developer.apple.com/documentation/gamekit/authenticating-a-player?language=objc)
- [Game Center entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.game-center)

## Write scope

- `platform/ios/ios-game/**`
- `docs/ios/game-center-status.md`
- this plan and package-local `link_check` example/import gate
- row 079 in the capability manifest, support-summary counts, and CI target/no_std steps
- the shared workspace dependency and lock entry for `objc2-game-kit` 0.3.2

Do not edit other capability rows, shared runtime APIs, authentication-handler properties, or any D43/D44 source in the shared checkout.

## Required implementation

- Add an iOS-only `ios-game` backend implementing D50 synchronously through the public generated GameKit binding.
- Read only `GKLocalPlayer.localPlayer` and `isAuthenticated`; map `true` to `Authenticated`, `false` to `NotAuthenticated`.
- Never set or read `authenticateHandler`, call deprecated `authenticateWithCompletionHandler`, trigger authentication initialization, display UI, install notifications/listeners, or retain the `GKLocalPlayer` beyond the read.
- Return only an owned Rust enum. Expose no player identifier, profile, native object, or GameKit dependency type outside the iOS crate.
- Require the signed app's Game Center entitlement `com.apple.developer.game-center` for a configured Game Center app. No privacy permission prompt or Game Center-specific `Info.plist` usage-description key is involved in this read. If the service has not been initialized, the raw Boolean may be false; this does not prove that no account exists.
- Treat the result as a momentary snapshot. Do not install `GKPlayerAuthenticationDidChangeNotificationName` observation or promise freshness.

## Validation and CI

- Run portable no-default check and strict Clippy for `framework-game`.
- Run device and simulator `cargo check` and strict all-target Clippy for `ios-game`.
- Build, but never execute, the package `link_check` example for device and simulator. `otool -L` must include GameKit/Foundation and exclude Swift runtime and unrelated capability frameworks.
- Add the portable crate to `cargo xtask no-std-check`; add device/simulator check, Clippy, and link-import steps to the macOS CI job.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`.
- Do not add or run tests, launch a simulator, sign an app, install an auth handler, or probe live Game Center state.

## Availability and evidence limits

The declaration-derived minimum iOS version is 4.1. Keep it separate from the installed SDK/deployment target. Compile/link evidence does not demonstrate a signed entitlement, current player state, Game Center account behavior, UI absence on a live device, or Apple parity. The Xcode 27.x repository baseline is not met by the inspected Xcode 26.6 host.
