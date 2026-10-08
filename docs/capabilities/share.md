# Outgoing share contract

`framework-sharing` defines portable values and a statically selected backend contract for one
outgoing share operation. It provides a backend-reported availability query but does not present
UI or define platform presentation behavior. Lifecycle, privacy, permission, and presentation
context requirements belong in a platform backend guide.

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
used the content. `ShareOutcome::Dismissed` means the backend reported that the user dismissed or
cancelled the operation. No platform activity identifier or recipient detail is exposed.
`ShareError` preserves the framework `ErrorKind` and optional signed native error code.

## iOS backend

The `ios-sharing` crate's `IosShareBackend` uses UIKit `UIActivityViewController` for owned text
and URL-text items. The caller supplies a live `UIViewController`, a `UIView`, and a `CGRect`
anchor; the presenter and source view must belong to the same window. Construct, poll, and drop the
backend on the main thread. iPad presentation uses the explicit popover anchor; other device idioms
use modal presentation.

The effective API floor is iOS 8.0.

The backend rejects file URLs, and accepted URL text may be normalized by the linked OS. Known
invalid presentation contexts return bounded errors, but UIKit's presentation call has no error
callback for an unforeseen refusal or race. Dropping an active future detaches Rust callback state
but does not dismiss native UI. No live share-sheet behavior, recipient delivery, or parity claim
is recorded. See the [iOS sharing guide](../ios/sharing.md) for the full API and evidence limits.

## Non-goals

V1 does not model file, image, or rich payloads; share extensions; recipients; activity selection;
previews; custom activities; clipboard access; or platform presentation behavior. Clipboard
read/write/clear is a separate contract in [`sharing.md`](sharing.md).
