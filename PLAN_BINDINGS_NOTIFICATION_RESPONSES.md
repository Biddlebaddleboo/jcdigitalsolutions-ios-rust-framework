# PLAN_BINDINGS_NOTIFICATION_RESPONSES.md — Workstream F4: Notification-Response C ABI

## Objective

Expose D9's owned notification-response values through an opt-in, capability-scoped C ABI. Reuse `framework-notifications`; add no delivery or delegate behavior

## Dependencies

- F1 core C ABI
- D9 `framework-notifications` response values

## Write scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs`
- `bindings/c/src/notification_responses.rs`
- `bindings/c/include/framework_notification_responses.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-notification-responses.sh`
- `Cargo.lock` local path-dependency edge only
- `PLAN_BINDINGS.md`
- `docs/bindings/notification-responses.md`
- `docs/DOCUMENTATION.md`
- `docs/VALIDATION.md`
- `.github/workflows/ci.yml`

Do not edit `crates/framework-notifications/**`, `platform/ios/ios-notifications/**`, B12 delegate code, the D9 contract, the core `framework.h` ABI, or the C++ binding

## Optional feature

- Cargo feature name `notification-responses`
- Optional path dependency `framework-notifications`
- Default `framework-c-api` build must not include `framework-notifications`
- Add no external crate or workspace dependency

## Exact C mapping

The capability file is `framework_notification_responses.h`. It includes `framework.h` and exposes the fixed-width `FrameworkNotificationResponseKind` tags

| C tag | Value | D9 kind | Required data |
| --- | ---: | --- | --- |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT` | 0 | `NotificationResponseKind::Default` | no action ID; no user text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS` | 1 | `NotificationResponseKind::Dismiss` | no action ID; no user text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION` | 2 | `NotificationResponseKind::CustomAction` | one action ID; no user text |
| `FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT` | 3 | `NotificationResponseKind::TextInput` | one action ID and user text; empty text is valid |

The notification ID and action ID use exact, case-sensitive UTF-8. Both must be non-empty and contain no NUL byte. No normalization, prefix, truncation, or action registration check is allowed. Text input uses exact UTF-8, may be empty, and may contain NUL bytes. Unknown tags or non-empty unused fields return `FRAMEWORK_STATUS_INVALID_ARGUMENT`

Input strings are borrowed only for the create call, then copied once into the owned D9 value. Used zero-length spans must have `data == NULL`; unused fields must be exactly `{NULL, 0}`

## C API

Declare an incomplete `FrameworkNotificationResponse` type and these symbols

```c
typedef uint32_t FrameworkNotificationResponseKind;

typedef struct FrameworkNotificationResponse FrameworkNotificationResponse;

typedef struct FrameworkNotificationResponseViewV1 {
    FrameworkNotificationResponseKind kind;
    uint32_t reserved;
    FrameworkStr notification_id;
    FrameworkStr action_id;
    FrameworkStr user_text;
} FrameworkNotificationResponseViewV1;

FrameworkStatus framework_notification_response_create(
    FrameworkStr notification_id,
    FrameworkNotificationResponseKind kind,
    FrameworkStr action_id,
    FrameworkStr user_text,
    FrameworkNotificationResponse **out_response);

FrameworkStatus framework_notification_response_get_view(
    const FrameworkNotificationResponse *response,
    FrameworkNotificationResponseViewV1 *out_view);

void framework_notification_response_destroy(FrameworkNotificationResponse **response);
```

Initialize `*out_response` to null before any fallible work. It must not point to a live response on entry. The view output is fully overwritten and its `reserved` field is zero

The view's `notification_id` is always present. `action_id` is present only for CustomAction and TextInput. `user_text` is present only for TextInput. Absent values use `{NULL, 0}`. Empty TextInput remains present by its kind tag, even when its span is `{NULL, 0}`. View spans borrow from the opaque response and remain valid only while that response stays alive

## Ownership and status rules

- The create function returns one opaque owned value; its original pointer must be destroyed once
- Destroy accepts a null pointer-to-pointer, a null handle, and clears a live original handle slot before drop
- Do not copy a live handle, destroy an alias, or use a view after destroy
- Input byte ranges must be readable for the call; output pointers must be aligned, writable, and distinct from live input and handle storage
- Invalid UTF-8, invalid IDs, malformed pointer/length pairs, unknown tags, invalid unused fields, or null required outputs map to `FRAMEWORK_STATUS_INVALID_ARGUMENT`
- Fallible string reserve failure maps to `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`
- C ABI entry points contain Rust panics and return `FRAMEWORK_STATUS_PANIC`; allocator abort policy remains the Rust process policy
- Destroy has no recoverable error path and never unwinds across C

## No-delivery limits

This slice creates, views, and destroys response values only. It adds no notification scheduling, delivery, authorization, delegate, callback, event queue, polling API, executor, category/action configuration, remote push, native response object, or B12 lifecycle semantics. No value implies that a notification was delivered or that a user action occurred

## Validation and handoff

- Check default-feature isolation and the opt-in dependency edge
- Build the feature-enabled static C library
- Compile and link generated C11 and C++17 API source files without execution
- Compare optional C symbols with the ABI manifest
- Run Rust/C++ formatting, docs index and zero-Swift checks, and `git diff --check`
- Do not add or run behavioral tests in this slice
- Report changed paths, commit SHA, non-test checks, exact mappings, ownership, deviations, and limits
