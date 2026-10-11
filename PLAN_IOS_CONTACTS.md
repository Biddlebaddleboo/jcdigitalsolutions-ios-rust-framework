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

## Additive increment B468 — bounded name search

B29 remains authorization-only; B462 remains one-record fetch by local identifier. B468 adds a
separate `ContactsSearchBackend` extension and `Contacts::search_contacts_by_name` over Apple's
public name predicate and synchronous `CNContactFetchRequest` enumeration. It rejects empty/NUL
queries, rechecks authorization for each valid call without prompting, accepts `Authorized` or
`Limited`, and stops at a caller-supplied `u32` result cap. Zero returns an empty vector after the
authorization check. The query passes unchanged; matching and result order stay native and have no
stronger promise. Search returns the shared owned common-field `Contact` snapshot, never an
unbounded all-contact array. The installed Xcode 26.6 / iPhoneOS 26.5 SDK and pinned
`objc2-contacts` 0.3.2 expose the needed APIs from iOS 9.0; registered device and Simulator floors
remain iOS 10.0 and iOS 14.0. Host tests and static compile/link gates do not prove live matching,
Limited visibility, native stop delivery, permission UI, or device runtime behavior.

## Additive increment B469 — add one new contact

B469 adds a separate source-compatible `ContactsCreateBackend` and `Contacts::create_contact`
operation over a portable Rust-owned `ContactDraft`. It creates exactly one record through public
`CNMutableContact` and `CNSaveRequest` APIs; it never updates or deletes an existing record. Each
valid request rechecks status without prompting and accepts only `Authorized` or `Limited`. Apple's
current Contacts access guide explicitly states that a Limited-access app can create contacts and
automatically access the contacts it creates; Limited access does not expose other contacts outside
the user's selected set. The request passes a nil container identifier, which Apple documents as
the default container. The synchronous operation returns no identifier because the public add and
save methods do not document a dependable post-save identifier result. It rejects an empty draft or
an empty phone/email value, preserves optional labels and supplied text, does not enforce phone or
email syntax, and maps native save errors to the platform category/code unless a fresh authorization
query shows access no longer permits the operation. The app supplies `NSContactsUsageDescription`.
The installed Xcode 26.6 / iPhoneOS 26.5 SDK headers and `objc2-contacts` 0.3.2 bindings expose the
required add APIs from iOS 9.0; registered deployment floors remain iOS 10.0 device and iOS 14.0
Simulator. Host tests and device/Simulator compile-link checks do not prove a live save, Limited
create behavior, default-container selection, permission UI, or an identifier result.

## Additive increment B471 — patch one existing contact

B471 adds `ContactUpdate`, `ContactsUpdateBackend`, and `Contacts::update_contact(identifier,
&ContactUpdate)` as a source-compatible patch extension. `None` preserves a field; an empty `Some`
string or phone/email vector deliberately clears it; a nonempty phone/email vector replaces that
full list. An all-`None` patch or an empty phone/email value is invalid. The iOS adapter validates
before native work, rechecks status without prompting, accepts only `Authorized` or `Limited`,
fetches one caller-selected identifier using the existing six common key descriptors, and creates
a public `CNMutableContact` copy. It sets only explicitly supplied common fields, then executes one
`CNSaveRequest.updateContact:` call. It does not create, delete, batch, or refetch a record.

Apple's current Contacts access guide says Limited access permits edit/delete only for contacts
currently granted to the app, while contacts created by the app are automatically accessible. The
caller-supplied identifier still has to resolve within that scope; no picker or prompt is added.
The public unified lookup may return a different unified identifier, so the API targets the single
logical contact returned by the lookup rather than a chosen account-level constituent. Contacts
error code 200 in `CNErrorDomain` maps to `NotFound`; a re-queried status that no longer permits
access takes precedence over a save failure, and other native codes are preserved.

The installed Xcode 26.6 / iPhoneOS 26.5 SDK headers mark the needed fetch, `mutableCopy`, mutable
field setters, and update-save APIs available from iOS 9.0. Pinned `objc2-contacts` 0.3.2 already
exposes `NSMutableCopying` to `CNMutableContact` and `CNSaveRequest::updateContact`; no dependency
or feature change is needed. Registered deployment floors remain iOS 10.0 device and iOS 14.0
Simulator. Host validation covers patch/error semantics and public facade dispatch; target compile
and link checks do not prove a live Contacts edit, Limited selection, or concurrent-store behavior.

## Additive increment B474 — delete one selected contact

B474 adds a source-compatible `ContactsDeleteBackend` and
`Contacts::delete_contact(identifier)` for one caller-selected local identifier. The iOS adapter
rejects empty/NUL identifiers before native work, rechecks authorization without prompting, accepts
only `Authorized` or `Limited`, and fetches only the supplied identifier with only
`CNContactIdentifierKey`. It creates a public `CNMutableContact` copy and executes one
`CNSaveRequest.deleteContact:` operation. It does not enumerate or search, delete groups, batch
records, or present UI. Contacts-domain code 200 maps to `NotFound` while access remains allowed;
a fresh denied/restricted/unknown status takes precedence over a native save error, and unrelated
native codes are preserved.

Apple's Contacts access guide says Limited access can delete contacts the person grants the app and
that records created by the app are automatically accessible. The caller identifier still must
resolve within the current accessible set. The SDK declares `CNSaveRequest` and `CNContact` mutable
copy support from iOS 9.0; objc2-contacts 0.3.2 already exposes `deleteContact:` using enabled
features, so no dependency change is needed. Registered floors remain iOS 10.0 device and iOS 14.0
Simulator. Host facade/error regressions and target compile/link selector checks do not prove a
live deletion, Limited-access behavior, Simulator execution, or physical-device runtime behavior.
