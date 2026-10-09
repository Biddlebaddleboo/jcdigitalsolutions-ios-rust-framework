# iOS Game Center status read

`ios-game` implements D50's local-player status query by synchronously reading the public GameKit `GKLocalPlayer.localPlayer.isAuthenticated` property. It returns only `LocalPlayerAuthenticationStatus`; it does not expose `GKLocalPlayer` or any player data.

## Use

```rust
use framework_game::{LocalPlayer, LocalPlayerAuthenticationStatus};
use ios_game::IosGameCenterBackend;

fn current_game_center_status() -> LocalPlayerAuthenticationStatus {
    let player = LocalPlayer::new(IosGameCenterBackend::new());
    player.authentication_status()
}
```

This is a direct Boolean property read, not an authentication operation. `localPlayer` obtains the shared object and may materialize a temporary offline player; that object access is distinct from Game Center authentication initialization. Apple documents initialization as setting `authenticateHandler`; Game Center may supply a view controller and show a brief initialization screen during that flow. B55 does not enter that flow: it does not set `authenticateHandler`, call deprecated `authenticateWithCompletionHandler`, show sign-in UI, register listeners, or request a permission prompt.

`GKLocalPlayer.localPlayer` may return a temporary offline player when no account is set up. `isAuthenticated == false` is therefore only a point-in-time negative value; it does not distinguish missing account from not-yet-initialized Game Center. An app that wants sign-in must implement a separate explicit user-facing authentication flow. B55 never performs it.

## Entitlement and availability

The consuming app must enable Game Center for a configured Game Center service and include the signed `com.apple.developer.game-center` entitlement. A missing entitlement or app configuration is not detected by this status read and may leave the reported Boolean false. This query has no Game Center-specific `Info.plist` usage-description key and does not ask for privacy authorization.

The local Xcode 26.6 build 17F113 / iPhoneOS 26.5 SDK declares `GKLocalPlayer` available from iOS 4.1. Its `localPlayer` property and `isAuthenticated` getter inherit that class floor; the convenience `local` property is iOS 13 and is not used. The iOS SDK does not annotate either selected getter with a main-thread requirement. The backend keeps the retained native object local to the synchronous call, returns only a Rust enum, and installs no callback. The result can become stale immediately; B55 does not observe `GKPlayerAuthenticationDidChangeNotificationName`.

## Binding and validation limits

The backend uses `objc2-game-kit` 0.3.2 with default features disabled and only `GKBasePlayer`, `GKLocalPlayer`, and `GKPlayer`; generated bindings expose `unsafe fn localPlayer() -> Retained<GKLocalPlayer>` and `unsafe fn isAuthenticated(&self) -> bool`. The adapter scopes both unsafe calls to the returned live retained object and scalar read. Target checks and the package import gate establish compile/link scope only. They do not exercise a real Game Center account, entitlement signing, auth transitions, or live UI behavior. The built link-check example is never executed.

Apple references: [`GKLocalPlayer`](https://developer.apple.com/documentation/gamekit/gklocalplayer), [`isAuthenticated`](https://developer.apple.com/documentation/gamekit/gklocalplayer/isauthenticated?language=objc), [`authenticateHandler`](https://developer.apple.com/documentation/gamekit/gklocalplayer/authenticatehandler?language=objc), [Authenticating a player](https://developer.apple.com/documentation/gamekit/authenticating-a-player?language=objc), and [Game Center entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.game-center).
