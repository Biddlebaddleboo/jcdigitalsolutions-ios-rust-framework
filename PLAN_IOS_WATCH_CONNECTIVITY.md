# PLAN_IOS_WATCH_CONNECTIVITY.md — Workstream B43: Watch Connectivity Support Query

## Objective

Implement D38 through the public iOS `WCSession.isSupported()` class method only.

## Bounded API

- Use `objc2-watch-connectivity` 0.3.2 with defaults disabled and only `WCSession`
- Preserve the iOS 9.0 API floor from the installed SDK
- Do not obtain or activate the default session, inspect pairing/app/reachability state, register delegates, prompt, or transfer data/messages
- Do not infer entitlements or full WatchConnectivity support from the Boolean result

## Validation

Run `platform/ios/ios-watch-connectivity/scripts/check.sh` for the source-surface guard, portable/iOS device/Simulator checks, strict Clippy, rustdoc, link/import, docs, zero-Swift-source, and diff evidence. The probe is linked, not run.
