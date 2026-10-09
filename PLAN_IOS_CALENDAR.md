# PLAN_IOS_CALENDAR.md — Workstream B30: iOS Calendar Authorization Backend

## Objective

Implement D25 through public EventKit APIs from Rust for iOS 17.0+ Calendar event authorization only.

## Dependencies

- Integrated `PLAN_FOUNDATION.md`
- `PLAN_CAPABILITIES_CALENDAR.md` and `framework-calendar`
- `objc2-event-kit` 0.3.2 generated `EKEventStore` and `EKTypes` bindings
- The installed Xcode iPhoneOS SDK headers

## Write scope

- `platform/ios/ios-calendar/**`
- `docs/ios/calendar.md`
- `PLAN_IOS_CALENDAR.md`
- `PLAN_VALIDATION_IOS_CALENDAR.md`
- `crates/framework-calendar/**` and `docs/capabilities/calendar.md` only for D25 portable dependencies

Do not edit root `Cargo.toml` or `Cargo.lock`, the canonical capability manifest, aggregate plans, CI, `tools/xtask`, documentation indexes, unrelated capability crates, C/C++ bindings, or Swift source.

## Required implementation

- Query event status with `EKEventStore::authorizationStatusForEntityType(EKEntityType::Event)`.
- Request full event access only with `requestFullAccessToEventsWithCompletion:`; never call deprecated `requestAccess(to:completion:)` / `requestAccessToEntityType:completion:`.
- Support iOS 17.0+ full event access and map `WriteOnly` separately from `FullAccess`; map unknown future raw statuses to `Unknown`.
- Start no prompt-capable operation during backend construction or future creation. Start the full-access request on first future poll.
- Treat the EventKit callback as arbitrary-queue work. Copy any error code into synchronized Rust-owned state, avoid accessing non-Send `EKEventStore` there, and re-query status after callback completion.
- Make future drop detach the result while keeping callback state safe through native completion. Complete at most once and contain Rust panics at the Objective-C block boundary.
- Require host app `NSCalendarsFullAccessUsageDescription` and document exact package API floor and runtime evidence limits.
- Use only public bindings with a package-local minimal feature set; keep the portable crate free of native dependency types.

## Explicit exclusions

No event/reminder fetch or enumeration, event edits or creation, write-only request, reminders authorization, EventKitUI, calendar UI, entitlements, Notes, Swift, live prompt, recipient choice, or device permission claim.

## Validation

Use `PLAN_VALIDATION_IOS_CALENDAR.md` for package-local host, iOS device/simulator, strict Clippy, format, docs, and diff gates. Record commands/results and separate compile/link evidence from live-device behavior.

## Status

Complete. `ios-calendar` uses `objc2-event-kit` `=0.3.2` with only `EKEventStore`, `EKTypes`, and `block2`, plus the `NSError` Foundation type. The local iPhoneOS 26.5 SDK confirms the request API floor is iOS 17.0 and marks the legacy request API deprecated from iOS 17. Host conversion/callback tests passed (4 tests); device and simulator `cargo check` and strict Clippy passed. Release import probes for both targets listed only EventKit.framework, Foundation.framework, `libSystem.B.dylib`, and `libobjc.A.dylib`; no Swift runtime import was present. No live prompt or device permission behavior was exercised.
