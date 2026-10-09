# PLAN_IOS_CONTACTS.md — Workstream B29: iOS Contacts Authorization

## Objective

Implement D24 authorization status and explicit access request through the public Contacts framework from Rust. No contact enumeration or data access operation is included.

## Dependencies

- Foundation A and iOS runtime B are integrated
- D24 `framework-contacts` is integrated
- Inspect the installed iOS SDK metadata and generated binding signatures before stating API availability

## Write scope

- `platform/ios/ios-contacts/**`
- `docs/ios/contacts.md`
- `PLAN_IOS_CONTACTS.md`

Do not edit the portable D24 crate, root workspace configuration or lockfile, capability manifest, aggregate indexes, CI, other iOS backends, Swift ABI, C bindings, or unrelated capability families.

## Required implementation

- Implement `ContactsBackend` with a caller-owned `CNContactStore` and no global service registry
- Query `CNContactStore::authorizationStatusForEntityType(CNEntityType::Contacts)` without prompting
- Call `requestAccessForEntityType_completionHandler` only when the request future is first polled
- Treat the request callback as arbitrary-queue work; re-query native authorization status inside the callback and return that status rather than inferring full access from the `granted` boolean
- Preserve `CNAuthorizationStatus::Limited` as portable `Limited`, distinct from `Authorized`
- Contain callback panics, retain callback state until one terminal result, detach future interest safely on drop, and do not unwind across an Objective-C block boundary
- Require the consuming host app to supply the exact `NSContactsUsageDescription` `Info.plist` key before a prompt-capable request
- Use only `objc2-contacts` 0.3.2 features needed for `CNContactStore` and blocks; keep binding types out of D24

## Availability and limits

- The installed Xcode 26.6 / iPhoneOS 26.5 SDK headers mark `CNContactStore`, the Contacts entity, status query, and access request as available from iOS 9.0
- The same SDK marks `CNAuthorizationStatusLimited` as introduced in iOS 18.0; report this separately from the authorization API floor
- The installed SDK is Xcode 26.6, not an asserted Xcode 27 baseline
- A Limited status means access to some contacts only. This backend reports authorization state and does not enumerate, fetch, identify, or mutate contacts

## Non-goals

- No contact enumeration/fetch, contact identifiers, contact edits, ContactsUI, picker/buttons, notes entitlement, Swift source, UI, or runtime contacts-data claim
- No permission prompt in construction, no global backend, and no executor requirement

## Validation and handoff

- Add pure status/error-conversion tests that do not show a live prompt
- Run iOS device and simulator `cargo check`, strict Clippy, format/docs/diff checks, and a package feature/linkage audit where a final link artifact is available
- Do not run a live permission prompt or claim simulator/device consent behavior
- Report changed files, exact commands, API floor and Limited introduction separately, linkage/features, runtime limits, deviations, and unresolved assumptions
