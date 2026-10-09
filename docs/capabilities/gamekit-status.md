# Local-player game-service status contract

`framework-game` defines an allocation-free `#![no_std]` value and static backend contract for a local player's point-in-time authentication status. The only implementation in this slice is the iOS [Game Center status read](../ios/game-center-status.md); this is not a general GameKit facade.

## Rust API

```rust
use framework_game::{LocalPlayer, LocalPlayerAuthenticationStatus};
use ios_game::IosGameCenterBackend;

fn game_center_status() -> LocalPlayerAuthenticationStatus {
    let player = LocalPlayer::new(IosGameCenterBackend::new());
    player.authentication_status()
}
```

`LocalPlayer<B>` owns the caller-supplied backend. `LocalPlayerAuthenticationBackend` selects a concrete backend type statically. Its availability value describes target API support; it does not query a player or validate signing entitlements. The synchronous status method needs no executor, callback, global registry, or hidden initialization. Portable values contain no Apple framework types, identifiers, or native objects.

## Status meaning

`Authenticated` means the backend reported an authenticated local player at the time of the call. `NotAuthenticated` means the backend reported false. It deliberately does not distinguish no account, signed out, user-declined sign-in, Game Center configuration failure, or a player whose Game Center authentication initialization has not occurred. In particular, this value is not evidence that an account does not exist.

The value is a one-time snapshot and can become stale immediately. D50 does not request authentication, initialize the service, subscribe to state changes, or represent player identity. GameKit-specific statuses are not user-permission grants.

## Scope

This contract covers no login, account creation, sign-in flow, identity/profile access, friends, invites, multiplayer, leaderboards, achievements, saved games, or Game Center UI. Only one iOS Game Center backend is included; other platforms have no implementation in this slice. See the [iOS guide](../ios/game-center-status.md) for the getter's side-effect boundary, entitlement, and evidence limits.
