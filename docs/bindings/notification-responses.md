# Notification-response C ABI

## Opt-in surface

The `framework-c-api` crate keeps this API out of its default build. Enable Cargo feature
`notification-responses` and include
[`framework_notification_responses.h`](../../bindings/c/include/framework_notification_responses.h)
alongside [`framework.h`](../../bindings/c/include/framework.h). The feature adds
`framework-notifications`, which uses `framework-core`. Rust apps can use the contract directly

All calls are synchronous and run on the C-selected thread. This API creates, views, and destroys
portable values only; it does not send notifications or model a native response object

## Kind map and text rules

| C tag | Value | Rust kind | Fields |
| --- | ---: | --- | --- |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT` | 0 | `NotificationResponseKind::Default` | no action ID or text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS` | 1 | `NotificationResponseKind::Dismiss` | no action ID or text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION` | 2 | `NotificationResponseKind::CustomAction` | one action ID, no text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT` | 3 | `NotificationResponseKind::TextInput` | one action ID and user text |

Notification and action IDs use exact, case-sensitive UTF-8. Each ID must be non-empty and contain
no NUL byte. No normalization, prefix, or truncation takes place; no check confirms action setup. Text
uses exact UTF-8; it may be empty and may contain NUL bytes. Unknown tags and non-empty unused fields
return `FRAMEWORK_STATUS_INVALID_ARGUMENT`

Create makes one copy of each used span into the Rust value. Input memory must stay valid for read and must
not change through that call. Every zero-length span must set data to null. Unused fields must
be exactly `{NULL, 0}`

## View and ownership

`FrameworkNotificationResponse` is opaque. Create requires an empty output slot, sets `*out_response`
to null before input checks, and returns one handle on success. Do not copy that handle. Destroy
takes its original slot, sets it to null before drop, accepts a null slot or null handle as a no-op,
and must run once for a live handle

`FrameworkNotificationResponseViewV1` has a `kind` tag, a zero `reserved` word, and three `FrameworkStr` spans

The `framework_notification_response_get_view` call sets the full view to zero before work, then writes all fields on success
The notification ID is always present. The action ID appears only for CustomAction and TextInput;
user text appears only for TextInput. Absent fields use `{NULL, 0}`. Empty TextInput text also uses
`{NULL, 0}`, while the tag still marks TextInput. View spans borrow from the handle and expire at
destroy

Do not race get_view with destroy. Do not use a view once destroy runs, destroy a copied handle, or call
destroy through an alias. Output slots must meet their C type's alignment and permit writes; keep
them distinct from input spans and live handle storage. The ABI cannot probe arbitrary C addresses or
prove that C code kept a handle alive

```c
FrameworkNotificationResponse *response = NULL;
FrameworkNotificationResponseViewV1 view = {0};
FrameworkStr notification_id = {(const uint8_t *)"upload-7", 8};
FrameworkStr unused = {NULL, 0};

FrameworkStatus status = framework_notification_response_create(
    notification_id,
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT,
    unused,
    unused,
    &response);
if (status == FRAMEWORK_STATUS_OK) {
    status = framework_notification_response_get_view(response, &view);
}
framework_notification_response_destroy(&response);
```

Malformed span pairs, invalid UTF-8, invalid IDs, unknown tags, non-empty unused fields,
and null required outputs map to `FRAMEWORK_STATUS_INVALID_ARGUMENT`. String reserve failure or a
view span length that does not fit in `u64` maps to `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`. A future
D9 response kind with no C tag maps to `FRAMEWORK_STATUS_INTERNAL_ERROR`; the view stays all zero

Handle allocation follows the process policy on allocation failure. Create and get_view catch Rust
panics and return `FRAMEWORK_STATUS_PANIC`; destroy has no recoverable status path

## No-delivery limits

This API has no schedule API, delivery path, authorization request, delegate API, callback API,
event queue, poll API, executor, category/action setup, APNs surface, or B12 lifecycle semantics. A value does
not prove that a notification reached a device or that a user took an action

`sh bindings/c/check-notification-responses.sh` checks feature isolation, Rust compile, C11 and
C++17 tag and layout values from the ABI manifest, view-field descriptions, status mappings,
output-slot preconditions, ownership text, static-library links, and header/archive C symbol names.
The gate checks exact manifest values; manual review must confirm that the ownership text matches
the implementation and this guide. It does not execute the C/C++ programs from the script or prove
runtime behavior
