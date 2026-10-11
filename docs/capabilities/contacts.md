# Portable Contacts authorization and read

`framework-contacts` defines a portable authorization contract and separate optional read-only
extensions for one-record fetch and bounded name search. It exposes no ContactsUI type or picker,
and does not enumerate all contacts or mutate the store. Existing `ContactsBackend` implementors
need not implement either operation; a backend opts in through `ContactsFetchBackend` or
`ContactsSearchBackend`.

## Authorization states

`ContactsAuthorization::Authorized` means the backend reports full access. `Limited` is separate:
it means the platform permits access to a user-selected subset, not all contacts. Both values make
`allows_contact_access()` true; only `Authorized` makes `is_full_access()` true. `Denied` and
`Restricted` do not permit access. `Unknown` is used for an unrecognized or unclassifiable state.

## Backend and request semantics

`Contacts<B>` owns the backend value supplied by its caller. The facade performs no global lookup,
hidden initialization, or executor selection; read methods allocate only the owned values in their
result contract. `authorization_status()` is a non-prompting query. `request_authorization()` is an
explicit operation; a native backend must defer its
prompt-capable call until the returned future is first polled. Dropping an unpolled future has no
prompt side effect. Dropping a started future abandons the result but cannot be assumed to dismiss
a system prompt. Callback state must not borrow future-owned storage and must remain valid until one
terminal native outcome. The first terminal completion becomes one future result; duplicate native
completions must not produce another result. A live future yields its result at most once.

## Read one record

`Contacts<B>::fetch_contact(identifier)` delegates to a `ContactsFetchBackend` and returns one
owned Rust snapshot. `Contact` contains the local identifier, given/family names, organization,
labeled phone numbers, and labeled email addresses. Names and organization are empty when absent;
labels are optional native label tokens, and the phone/email order is not a stable sorting promise.
The method is synchronous, and a platform store read can block. Avoid calling it from a
latency-sensitive UI context when the read may take time; the crate does not add a hidden executor.
The portable crate uses `no_std + alloc`; it does not expose Apple types.

```rust,ignore
use framework_contacts::Contacts;
use ios_contacts::IosContactsBackend;

fn read_selected_contact(identifier: &str) -> Result<framework_contacts::Contact, framework_contacts::ContactsError> {
    let contacts = Contacts::new(IosContactsBackend::new());
    contacts.fetch_contact(identifier)
}
```

Fetch never prompts or requests permission. It re-queries authorization for each call and proceeds
only for `Authorized` or `Limited`. The host app must include `NSContactsUsageDescription` for
Contacts access; this crate does not edit the host property list. Empty or NUL-containing
identifiers are invalid. A Contacts
`RecordDoesNotExist` result maps to `NotFound`; under `Limited`, an identifier outside the
user-selected subset can have the same result as a missing or stale identifier. Local identifiers
are not durable identities and may stop resolving after deletion or record unification. Contacts
may return a unified record whose identifier differs from the supplied identifier.

The fetch path reads only the six declared keys and returns owned values detached from the native
contact object. It does not observe later store changes. The iOS backend requires the host app's
`NSContactsUsageDescription` for Contacts access; fetch itself does not request access. Static
checks do not prove permission state, selected-contact visibility,
or a live Contacts fetch on simulator or device.

## Bounded name search

`Contacts<B>::search_contacts_by_name(query, max_results)` delegates to the separate
`ContactsSearchBackend` extension and returns at most the caller's `u32` limit of the same owned
`Contact` snapshots. Empty or NUL-containing queries return `InvalidSearchQuery`; valid queries
are passed unchanged to the backend. On iOS, search uses Apple's public
`CNContact::predicateForContactsMatchingName` with synchronous `CNContactFetchRequest`
enumeration. Apple documents that the name may contain any number of words, but this API does not
promise substring, case, token-order, or result-sort behavior. Result order follows native callback
delivery and is not a stable contract.

Every valid call rechecks authorization and proceeds only for `Authorized` or `Limited`; search
never requests access. A zero limit returns an empty vector after that check and without native
enumeration. For a nonzero cap, the iOS callback stops as soon as it reaches the limit and keeps no
more than that many snapshots. Limited access searches only the user-selected visible subset. A
native enumeration failure rechecks authorization and otherwise returns `Backend`; no matches are
a successful empty result, not `NotFound`.

Search is synchronous and Contacts fetches perform I/O, so avoid it on latency-sensitive UI paths.
The host app still supplies `NSContactsUsageDescription`; the crate adds no executor or prompt.
Host tests and iOS device/Simulator compile-link checks do not prove live matching, Limited
visibility, native callback stop delivery, or device runtime behavior. See Apple's
[`predicateForContacts(matchingName:)`](https://developer.apple.com/documentation/contacts/cncontact/predicateforcontacts%28matchingname%3A%29).
