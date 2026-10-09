# PLAN_VALIDATION_IOS_GAMEKIT.md — G49: Game Center Local-Player Status

## Objective

Gate D50/B55's portable point-in-time authentication-status contract and the iOS
`GKLocalPlayer.localPlayer.isAuthenticated` read.

## Required checks

- Run `sh platform/ios/ios-game/check.sh` on macOS.
- Check `framework-game` without default features, strict Clippy, and rustdoc; check `ios-game` for
  device and Simulator targets with strict Clippy.
- Enforce that the adapter reads only the local player and its authentication Boolean; reject auth
  handlers, deprecated auth methods, listeners, player identity, and UI.
- Build, but do not execute, device and Simulator link probes; require GameKit/Foundation and reject
  Swift runtime or unrelated capability frameworks.
- Run no-std, formatter, documentation, zero-Swift-source, and diff checks. Do not add or run tests,
  launch a simulator, sign/install an app, or initiate authentication.

## Evidence boundary

Compile/link checks do not establish signed entitlement configuration, live player/account state,
authentication transitions, or UI behavior. The inspected Xcode 26.6 / SDK 26.5 host is below the
planned Xcode 27.x baseline.
