# Portable Contacts authorization and read

`framework-contacts` defines a portable authorization contract and an optional read-only fetch
extension. It exposes no ContactsUI type or picker, and does not enumerate, search, or mutate the
contact store. Existing `ContactsBackend` implementors need not implement record fetch; a backend
opts in through `ContactsFetchBackend`.

## Authorization states

`ContactsAuthorization::Authorized` means the backend reports full access. `Limited` is separate:
it means the platform permits access to a user-selected subset, not all contacts. Both values make
`allows_contact_access()` true; only `Authorized` makes `is_full_access()` true. `Denied` and
`Restricted` do not permit access. `Unknown` is used for an unrecognized or unclassifiable state.

## Backend and request semantics

`Contacts<B>` owns the backend value supplied by its caller. It performs no global lookup, hidden
initialization, allocation, or executor selection. `authorization_status()` is a non-prompting
query. `request_authorization()` is an explicit operation; a native backend must defer its
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
