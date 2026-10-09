# Portable Contacts authorization

`framework-contacts` defines only a portable authorization contract. It does not expose a contact
record, identifier, enumeration or fetch operation, write operation, ContactsUI type, or picker.
Having `Authorized` or `Limited` status therefore does not mean this crate can read contact data.

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
a system prompt. Native callback state must remain safe until one terminal result, and a live
future yields at most one result.

The D24 contract covers authorization status/request only. It makes no claim about enumeration,
fetch, contact fields, identifiers, writes, selection UI, or access to any particular contact.
