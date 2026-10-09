# Outgoing share contract

`framework-sharing` defines portable values and a statically selected backend contract for one
outgoing share operation. Its backend-reported, non-presenting availability query may return
`Availability::Unknown` and does not guarantee that a particular request can be presented or
completed. This contract does not present UI or define platform presentation behavior. Lifecycle,
privacy, permission, and presentation context requirements belong in a platform backend guide.

## Items and ownership

V1 supports owned UTF-8 text and URL text through `ShareItem::Text` and `ShareItem::Url`. A
`ShareRequest` always contains at least one item, retains item order, and may contain multiple
items. Construction takes ownership of each `String`; the request stores items in an allocated
`Vec`. The backend receives ownership of the request and may need additional copies to create and
retain native items. The portable facade makes no implicit payload copy.

URL text is passed to the backend exactly as stored and is not validated, parsed, or normalized by
the portable contract. A backend may reject, canonicalize, or otherwise transform URL text while
creating native items; exact URL-string preservation is not promised.

## Operation and result

`ShareClient<B>` owns caller-provided backend state selected statically through `ShareBackend`.
Availability is a non-presenting query. The API adds no boxed trait object, `Send` bound, executor,
global registry, or hidden initialization. Backends can be substituted at compile time by choosing
a different concrete `B`.

The `share` operation starts on its first future poll. Dropping it before that poll starts no
backend work. Once native presentation begins, dropping the Rust future suppresses its result and
requires the backend to detach callback state safely and suppress late delivery. The backend must
settle the future exactly once while it remains attached, make late or duplicate native callbacks
inert, and release callback state exactly once. Rust future drop does not guarantee cancellation or
dismissal of native work; system share UI may remain visible.

`ShareOutcome::Completed` means the backend reported completion, not that a recipient received or
used the content. `ShareOutcome::Dismissed` means the backend reported that the operation was
dismissed or cancelled without completion; it does not identify an actor or platform-specific
cause. No platform activity identifier or recipient detail is exposed.
`ShareError` preserves the framework `ErrorKind` and optional signed native error code.

The iOS backend, native presentation requirements, and evidence limits are documented in the
[iOS sharing guide](../ios/sharing.md).

## Non-goals

V1 does not model file, image, or rich payloads; share extensions; recipients; activity selection;
previews; custom activities; clipboard access; or platform presentation behavior. Clipboard
read/write/clear is a separate contract in [`sharing.md`](sharing.md).
