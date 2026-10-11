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

## B30 explicit exclusions

At B30 completion this workstream excluded event/reminder reads or enumeration, event creation/edits, write-only request, reminders authorization, EventKitUI, calendar UI, entitlements, Notes, Swift, live prompt, recipient choice, and device permission claims. B465 and B481 are separate later additive scopes and do not revise B30's history.

## Validation

Use `PLAN_VALIDATION_IOS_CALENDAR.md` for package-local host, iOS device/simulator, strict Clippy, format, docs, and diff gates. Record commands/results and separate compile/link evidence from live-device behavior.

## Status

B30 complete at its original scope. It used `objc2-event-kit` `=0.3.2` with `EKEventStore`, `EKTypes`, and `block2`, plus the `NSError` Foundation type. The local iPhoneOS 26.5 SDK confirmed the request API floor is iOS 17.0 and marks the legacy request API deprecated from iOS 17. Host conversion/callback tests passed (4 tests); device and simulator `cargo check` and strict Clippy passed. Release import probes for both targets listed only EventKit.framework, Foundation.framework, `libSystem.B.dylib`, and `libobjc.A.dylib`; no Swift runtime import was present. No live prompt or device permission behavior was exercised.

## B465 — bounded Calendar event-range read

B465 is a separate additive scope; it does not revise B30's authorization-only history or exclusions. `framework-calendar` now models finite Unix-second event times, ordered ranges, and owned event summaries with optional native fields. `ios-calendar` uses the existing `EKEventStore` and public EventKit range predicate plus synchronous `eventsMatchingPredicate:`. The read rechecks current status and requires `FullAccess`; it never prompts, and it rejects a range longer than 1461 days rather than accept EventKit's four-year truncation. All-day timestamps remain EventKit `NSDate` instants plus an explicit `is_all_day` flag; EventKit uses the default time zone. The synchronous call may block and creates no thread/executor. Host tests and registered target/link/import/deployment gates are tracked in B465 evidence. Static checks do not prove live permission state, returned calendar contents, or device runtime.

## B481 — writable calendar selection and one-event creation

B481 is a separate additive scope; B30 remains authorization-only and B465 remains read-only. `Calendar::writable_event_calendars` requires existing `FullAccess`, snapshots only EventKit calendars whose entity mask includes events and whose current content is modifiable, and returns Rust-owned identifier/title values without prompting or UI. `CalendarEventDraft` preserves caller text, finite ordered Unix-second bounds, optional location/notes, and all-day choice. `Calendar::create_event` re-resolves the selected opaque calendar identifier, rechecks `FullAccess`, event support, and writability before creating one `EKEvent`, then invokes one synchronous committed save. It returns no EventKit identifier because that identity can change after sync or calendar moves. Event create/update/delete beyond this one-event addition, reminder access, and calendar UI remain out of scope. Final `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-calendar` reported 75 PASS, 0 FAIL, 0 BLOCKED, and 2 runtime SKIPPED; simulator execution and physical-device proof remain unverified. The host test gate passed all 14 tests (10 existing unit tests and 4 focused B481 integration tests). Static host/target/link checks do not prove live permission, calendar selection, event persistence, or device behavior.

## B483 — delete one selected event

B483 adds a source-compatible delete backend and `Calendar::delete_event(&CalendarEvent)`. The portable facade rejects a missing/empty identifier before backend dispatch. iOS requires current `FullAccess` without prompting, resolves only the selected ID through public `eventWithIdentifier:`, checks the returned ID and exact selected start/end instants, then verifies the attached calendar supports Event and currently allows modifications. It calls public `removeEvent:span:commit:error:` exactly once with `EKSpan::ThisEvent` and commit enabled. The exact date check is required because Apple documents `eventWithIdentifier:` as returning the first occurrence for a recurring identifier; a mismatch fails closed as `EventNotFound`. The API does not enumerate or search, access reminders, retry, remove a series, or use EventKitUI. Post-commit `cargo +1.94.1 test --locked -p framework-calendar -p ios-calendar` passed 25 tests, including a recurring-first-occurrence mismatch/no-removal regression. The registered `ios-calendar` profile passed 100 checks and skipped only Simulator execution and physical-device runtime; iOS 17 device/Simulator compilation, Clippy, Rustdoc, selector/link/import gates, source guards, and docs passed. These static checks do not prove live deletion or EventKit database state.
