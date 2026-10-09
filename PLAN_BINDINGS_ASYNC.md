# PLAN_BINDINGS_ASYNC.md — Workstream F8: Capability-Scoped Async C ABI

## Status

F8 is closed by design disposition after a source and contract audit. F5 and F7 supply two distinct capability-owned async forms. Neither needs a shared C async API, operation registry, executor, callback table, or universal handle

F5 uses a per-client C handle with durable D10 task IDs and snapshot queries. F7 uses one B7 `IosShareSession` C handle with one active operation and a host callback. Their handle and callback rules match their backend contracts

## Goal

Set the common async ABI rules without force of one cross-capability shape

`FrameworkOperationHandle` and `FrameworkCompletionCallback` are F1 type declarations only; F1 exports no operation start, cancel, destroy, or universal registry function

- Each capability owns its opaque client/session handle or its durable ID
- State the start acceptance point and all sync rejection rules
- State the result path, callback count and trigger, and callback thread
- State who owns callback context and how long it must stay live
- State cancel effect, destroy/drop result, handle lifetime, and native/error map
- A callback is not required when durable status or snapshot queries provide the result path
- Claim exactly-once delivery only when the backend contract supports it; otherwise state the trigger and how the host ends callback lifetime
- Add no global operation map, executor, cross-capability handle type, or shared callback table

## F5 transfer evidence

- `framework_ios_transfer_client_create` yields one unique main-thread client handle for one B13 `IosTransferBackend`; no process-global map exists
- `framework_ios_transfer_start_download` uses a host-assigned nonzero `FrameworkTransferIdV1` that maps to D10 `TransferId`; D10 saves acceptance before return, and duplicate IDs reject with `FRAMEWORK_STATUS_ALREADY_EXISTS`
- `framework_ios_transfer_status` returns an owned snapshot or `out_found == 0`; task state has no per-task callback
- `framework_ios_transfer_cancel` records a durable stop request. Status may stay queued or active, and a terminal native result may win the race. `framework_ios_transfer_forget` removes terminal state only
- The host event callback is a separate UIKit session-drain signal, not a task result. On
  acceptance, F5 stores the callback and context address; it does not own or free host context
  memory. A rejected forward stores neither. B13 calls an accepted callback once on the main queue
  after URLSession finish-events and initial reconciliation; the host retains its context and
  client through that call. Early client drop releases the retained callback without a call, so
  the host must keep the client alive
- `framework_ios_transfer_finish_launch_without_background_events` is the alternate ordinary-launch path; call one launch path only
- Client destroy clears the original slot before drop and does not cancel or forget durable tasks. Keep the client alive through each accepted event callback. Snapshots may outlive the client
- `TransferError` maps through `FrameworkStatus` with an optional native-code output. `BackgroundEventError::SessionIdentifierMismatch` and `LaunchAlreadyClassified` map to `FRAMEWORK_STATUS_INVALID_ARGUMENT`; `CompletionAlreadyPending` maps to `FRAMEWORK_STATUS_ALREADY_EXISTS`
- B13 and F5 document main-thread use, non-exhaustive fallbacks, and panic containment

Evidence paths: `PLAN_BINDINGS_TRANSFER.md`, `PLAN_IOS_BACKGROUND_TRANSFER.md`, `bindings/c/src/ios_transfer.rs`, `platform/ios/ios-transfer/src/platform.rs`, `docs/bindings/ios-transfer.md`, and `docs/ios/transfer.md`

## F7 share evidence

- `framework_ios_share_session_create` yields one unique main-thread session handle with retained UIKit context and at most one active operation; no process-global lookup exists
- `framework_ios_share_start` copies and validates the request before it stores the callback and
  context address. A rejected start returns a status and stores neither; the host owns context
  memory
- `FRAMEWORK_STATUS_OK` means B7 requested UIKit presentation. It does not prove visible UI or a terminal result
- If UIKit reports a terminal result, B7 delivers one callback on main after `start` returns. B7
  suppresses duplicate native callbacks. UIKit has no presentation-error callback, so the host
  must not rely on terminal delivery; the host keeps context memory live until a result, successful
  cancel, or session destroy
- Cancel detaches callback state but does not guarantee UI dismissal. Off-main destroy returns unavailable and leaves the slot as-is; on main, destroy clears the original slot before drop and detaches active callback state
- Known `ShareError::Backend` values map through `FrameworkStatus::from_error`; unknown variants map to `FRAMEWORK_STATUS_INTERNAL_ERROR`. A nonzero `NSError.code` is kept only if it fits `int32_t`. B7 contains Rust callback panic; C callbacks must not unwind into Rust
- Create, start, cancel, destroy, and callback use stay on main. The host must serialize each session and keep callback context live until callback, successful cancel, or destroy

Evidence paths: `PLAN_BINDINGS_SHARE.md`, `PLAN_IOS_SHARE.md`, `bindings/c/src/ios_share.rs`, `platform/ios/ios-sharing/src/share_platform.rs`, `platform/ios/ios-sharing/src/share_operation.rs`, `docs/bindings/ios-share.md`, and `docs/ios/sharing.md`

## Gate and validation evidence

F5's recorded `sh bindings/c/check-ios-transfer.sh` pass covers feature isolation, manifest/header checks, device/simulator compile and link, exact probe imports, and forbidden-symbol scans; it did not run probes or tests

F7's recorded `sh bindings/c/check-ios-share.sh` pass covers feature isolation, manifest/header/archive checks, device/simulator links, direct imports, and forbidden-symbol scans; it did not run probes or tests

This F8 audit reads the source and contract paths above. It ran no Cargo command, build, link, test, or consumer. A scoped `git diff --check` and whitespace scan are the only F8 edit checks

## Acceptance

- F5 and F7 each use capability-owned state without a universal C operation registry
- Each capability documents start acceptance, handle or ID lifetime, result path, callback rule, cancel effect, destroy/drop rule, thread, and error map
- F5's durable task status does not claim a per-task exactly-once callback
- F7's one-shot result callback does not claim a result when UIKit sends none
- New async C capabilities must meet the common rules above before their ABI plan can pass review
