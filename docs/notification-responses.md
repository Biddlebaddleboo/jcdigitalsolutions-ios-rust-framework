# Notification response values

`framework_notifications::response` holds owned values for the kind and data of a local-notification interaction. It does not deliver or observe responses. Native delivery and callback behavior remain a separate B12 workstream.

## Rust API

```rust
extern crate alloc;
use alloc::string::String;
use framework_notifications::NotificationId;
use framework_notifications::response::{
    NotificationActionId, NotificationResponse, NotificationResponseKind,
};

let response = NotificationResponse::new(
    NotificationId::new(String::from("daily-reminder")).expect("valid notification ID"),
    NotificationResponseKind::TextInput {
        action_id: NotificationActionId::new(String::from("reply")).expect("valid action ID"),
        text: String::from("I'll review it"),
    },
);
# let _ = response;
```

The response owns the existing D3 `NotificationId` and one `NotificationResponseKind`:

- `Default` represents the standard/default activation and has no extra payload.
- `Dismiss` represents dismissal and has no extra payload.
- `CustomAction` owns a `NotificationActionId`.
- `TextInput` owns a `NotificationActionId` and the submitted text.

`NotificationActionId` is exact, case-sensitive UTF-8. Its constructor rejects empty values and NUL bytes, with no normalization, prefixing, truncation, or length bound. It can represent an action identifier supplied by a native layer; this module does not configure or verify that action.

Text input is an owned `String`, so its bytes are valid UTF-8. Empty text is allowed. The value is preserved as supplied: no trimming, normalization, or NUL rejection occurs. `NotificationResponse::new` takes ownership of its validated notification ID and response kind. Accessors borrow their values; `into_parts` transfers both values without a response-layer copy. Cloning an ID, action ID, or response uses ordinary owned-string clone semantics.

## Boundary and limits

These are data values only. They do not add a response delegate, event queue, callback, polling operation, backend method, lifecycle, delivery guarantee, authorization API, or executor. They expose no `UNNotificationResponse`, system action constant, category, or action-definition type. `Default` and `Dismiss` are portable semantic variants; custom action IDs are passed through exactly.

The values do not imply response delivery, validate that an action was registered, or say when or on which thread a native response arrived. B12 owns those delivery and threading semantics. Remote push/APNs/PushKit and category/action configuration remain out of scope.

`NotificationActionId` validation errors use `NotificationResponseError::InvalidActionIdentifier`, categorized as `ErrorKind::InvalidInput`. Notification IDs retain D3's existing `NotificationError::InvalidIdentifier` validation at their construction boundary.

The response module remains `no_std` and uses `alloc` only for owned action identifiers and text. It adds no dependency or runtime state.
