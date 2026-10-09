# Portable durable HTTP downloads

`framework-transfer` defines GET file downloads over a static backend. It uses `no_std` + `alloc` and starts no executor or global registry

The control API is synchronous and has no operation future or callback. The backend retains
background task state; clients observe it through `status`

## Request

`TransferId` is a nonzero app-assigned `u128`. Retain the exact ID across app relaunch. A durable ID conflict never replaces an old task; reuse is valid only after terminal `forget`

`DownloadRequest` borrows a validated `framework_network::HttpUrl`, an ordered slice of `Header` values, and a destination `framework_files::AppPath`. The request is GET-only and has no method override or body

At successful `start_download` return, the backend owns a durable task record and the request data it needs. Success means task acceptance, not immediate network activity. A new backend instance can query the last durable task state after app relaunch

`TransferError::DuplicateId` leaves the existing task unchanged. Any other `start_download` error means this call accepted no new task record. Outcomes after acceptance are observed through `status`; a retained terminal failure uses `TransferStatus::Failed`

`HttpUrl` checks the HTTP(S) scheme, authority presence, and forbidden ASCII whitespace/control bytes; it does not do full URL parse or normalization. `Header` checks a token name and field bytes, retains opaque values, and preserves app order and duplicate names. `AppPath` keeps a semantic app directory and relative path; it does not prove sandbox containment or resolve symlinks

## Status and cancellation

`TransferSnapshot` holds the stable ID and last durable `TransferStatus`: `Queued`, `Active`, `Succeeded`, `Failed`, or `Cancelled`. A status query after relaunch returns the last retained state, including terminal metadata until `forget` removes the record, or `None` if no task record has that ID

`Succeeded` holds the HTTP `StatusCode` and owned `ResponseHeader` values. Non-2xx status remains an ordinary completed HTTP result. The response body is at the destination path; D10 does not return body bytes

The portable header vector keeps its supplied order and duplicate values. A native API may expose a dictionary or normalized view instead of wire order or duplicates. Such a backend must document its observed header limits and must not claim wire-level fidelity

`cancel` records a durable stop request only; it does not prove task stop or destination change. Status may remain `Queued` or `Active` until a terminal result. A terminal task is a no-op success, and a terminal result may win a cancel race. `forget` accepts terminal tasks only and removes the durable task record, not a completed file. Forgetting is the only way to free an ID for reuse

## Errors

`TransferError::DuplicateId`, `TransferError::NotFound`, and `TransferError::NotTerminal` map to `ErrorKind::AlreadyExists`, `ErrorKind::NotFound`, and `ErrorKind::InvalidInput`, respectively, and carry no platform code. `TransferError::Backend(error)` preserves `error.kind()` and `error.platform_code()`. `status` reports an absent ID as `Ok(None)`; `cancel` and `forget` report an absent ID as `TransferError::NotFound`, while forgetting a queued or active task returns `TransferError::NotTerminal`

`TransferStatus::Failed(error)` retains the `framework_core::Error`, including its `ErrorKind` and optional `PlatformErrorCode`, in the durable snapshot. A backend error therefore remains available after relaunch until the record is forgotten

## Atomic destination

A backend stages a download away from its final path, then commits the full file atomically. Before commit, readers see the whole prior file or no file; after commit, readers see the whole new file. Partial new bytes never appear at the destination. A failed or cancelled task leaves the prior whole file or no file at the destination

Atomic commit does not promise crash durability. The backend must enforce app-sandbox containment and its symlink policy. A backend that cannot meet durable task or atomic destination guarantees reports `Availability::Unsupported`; a request-specific limit may return `framework_core::ErrorKind::Unsupported` before task acceptance

D10 has no upload, method override, inline body, progress stream, retry, redirect, cookie, response body, permission, or native-platform policy. B13 implements a separate iOS backend; B14 owns its `ios-files` temp-file adoption operation. See the [iOS background-transfer guide](../ios/transfer.md) for session lifecycle, event forwarding, header normalization, and the ambiguous commit-crash limit
