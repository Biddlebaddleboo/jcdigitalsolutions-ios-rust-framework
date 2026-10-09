# PLAN_IOS_ACCESSORY.md — Workstream B44: ExternalAccessory Presence Query

## Objective

Implement D39's scalar current-list query through the public ExternalAccessory API.

## Bounded API

- Call only `EAAccessoryManager.sharedAccessoryManager()`, `connectedAccessories`, and array `count()`
- Map zero to `AccessoryPresenceSnapshot::NoneAvailable` and nonzero to `AccessoryPresenceSnapshot::OneOrMoreAvailable`
- Use `objc2-external-accessory` 0.3.2 with defaults disabled and only `EAAccessory` / `EAAccessoryManager` features
- Record the SDK's iOS 3.0 declaration floor
- Do not present a picker, register notifications, inspect protocol strings, create `EASession`, access streams, or communicate
- Do not infer host protocol metadata, MFi eligibility, entitlement, privacy, or physical attachment from the scalar result

## Validation

Run `platform/ios/ios-accessory/scripts/check.sh` for portable and iOS target checks, strict Clippy, rustdoc, source-surface guard, and Release import/symbol audit. Link evidence does not establish live accessory behavior.
