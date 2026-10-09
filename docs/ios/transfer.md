# iOS background transfers

`ios-transfer` implements D10's durable GET-download contract with Foundation background
`URLSession`. The app owns its `UIApplicationDelegate`; this crate does not import UIKit or set a
delegate on the app's behalf.

## Setup and launch routing

Create `IosTransferBackend` on the main thread with a stable, app-unique session identifier and a
`MainThreadMarker`. Keep it on the main thread and retain it for the app lifetime while tasks or
background events remain active. The backend is `!Send` and `!Sync`; its sync file and journal calls
may block the main thread. Do not share the identifier with another URLSession owner.

In `application(_:handleEventsForBackgroundURLSession:completionHandler:)`, the app delegate must
first retain UIKit's completion handler in app-owned state. It can then create/reattach the backend
with the supplied stable session identifier and forward the retained handler through
`handle_background_events` (or the C-owned callback/context form). This order ensures the app does
not lose the handler while constructing the session. The URLSession finish-events callback may
arrive before the backend receives the handler; the backend preserves that early event until it can
consume the forwarded handler.

After UIKit launch routing, choose one path:

- When UIKit calls
  `application(_:handleEventsForBackgroundURLSession:completionHandler:)`, pass the identifier and
  completion block to `handle_background_events`.
- On an ordinary launch with no such UIKit callback, call
  `finish_launch_without_background_events` so an absent active task can become a terminal failure
  after initial session reconciliation.

The backend copies and retains the completion block. It waits for both the initial task-list
reconciliation and `URLSessionDidFinishEventsForBackgroundURLSession`, then consumes the handler
once via `DispatchQueue::main().exec_async`. Apple may call the URLSession finish-events delegate
on a secondary queue; UIKit requires its system completion handler on main. Keep the backend alive
until the handler runs. Do not use backend drop as an event-completion signal.

`MainThreadBound` uses synchronous main dispatch when it is dropped away from main and may deadlock
if the main runloop is not running. The normal event path avoids that case: it moves the bound block
into a `Send` main-queue closure, extracts and calls it there, then drops it on main. The backend is
`!Send`, requires a main-thread marker at construction, and must be dropped on main. Its `Drop`
clears any pending retained block on main without invoking it; therefore the app must keep the
backend alive through the UIKit completion call, and must not rely on drop to finish a UIKit event.

For a C-owned host, `unsafe handle_background_events_c` wraps an
`unsafe extern "C" fn(*mut c_void)` and context in the same completion path. The app must retain the
backend and keep the context valid until the one call on main; the callback must not unwind. This
seam does not add `framework-c-api` or transfer UIKit ownership to Rust.

## Durable state and task identity

Records live at:

```text
ApplicationSupport/ios-transfer/<session-identifier>/<32-hex-transfer-id>.record
```

New private, versioned binary records begin with `JCDITF02`, have a 1 MiB size cap, and store the
nonzero `TransferId`, optional app-owned task-incarnation token, URL, request headers, semantic
destination, cancel-request bit, last status, an optional pre-commit intent with
`StatusCode`/`ResponseHeader` metadata, or terminal metadata and portable/native error codes. The
decoder accepts earlier `JCDITF01` records without a task token. `ios-files` writes each journal
file with atomic replacement. The record contains URL and header values in the app sandbox and is
not encrypted by this crate.

Native tasks use `taskDescription` values of the form
`jcd-ios-transfer-v2:<32 lowercase hex TransferId>:<32 lowercase hex incarnation>`. B13 generates a
nonzero `NSUUID` incarnation, persists it in `JCDITF02`, and mirrors it in the task description.
This app-owned token identifies the task across relaunch and lets a valid delegate callback be
checked before asynchronous initial reconciliation completes. `NSURLSessionTask.taskIdentifier`
is unique within a live session and is used only in an in-memory map rebuilt by the task snapshot
and populated at task creation; B13 does not assume this native identifier stays stable across
relaunch.

On construction, `getAllTasksWithCompletionHandler` reattaches only a unique task whose transfer
tag and incarnation match its journal record, resumes matching suspended tasks, and creates a
native task for each durable queued record with no match. Legacy `JCDITF01` records and `v1` task
tags have no incarnation proof: B13 cancels those native tasks rather than risk attaching a stale
task to a reused ID. A queued legacy record is resubmitted with a new incarnation; an absent legacy
active record resolves to `Failed(Unavailable)` (or `Cancelled` if cancellation was durably
requested), while a legacy commit-intent record stays `Active`.

Each new native task receives a fresh `NSUUID` token; B13 creates one task per token, persists the
token before resume, and generates a new token before queued resubmission. Thus duplicate tasks for
one `TransferId` have distinct tokens under backend control. Before reconciliation, only the task
whose token matches the journal record may act; reconciliation later cancels all tasks in the
duplicate group. A duplicate with the same token would require UUID collision or external task
mutation, which is outside the guarantee; reconciliation still cancels that group.

A matching early `didFinishDownloadingToURL` callback may persist a commit intent and adopt the file
before reconciliation discovers a duplicate group. If reconciliation then clears the task
descriptions before `didCompleteWithError`, that final callback cannot be correlated. Reconciliation
then runs missing-task finalization: if this process still holds proof that B14 returned `Atomic`,
it may persist `Succeeded` from that proof before the final callback; without that proof (for
example, a recovered intent), the record remains `Active`. A matching final callback that runs
before reconciliation may also persist `Succeeded`, but never without live atomic-adoption proof.

While the initial task snapshot is pending, `start_download` returns `Unavailable` without
accepting an ID. This prevents a stale snapshot from classifying a newly accepted active transfer
as missing. Reconciliation groups tasks by transfer tag. If a tag has multiple native tasks, B13
clears the tag and cancels every task in that ambiguous group; it does not guess which incarnation
owns the durable record. It also clears and cancels tasks for unknown or terminal records, and
cancels untagged orphan tasks in this dedicated session.

`didFinishDownloadingToURL`, `didCompleteWithError`, and asynchronous cancel requests must match
the record's app-owned task incarnation and its task-description token. If the live task map already
has an entry, its `taskIdentifier` must also match; an absent map entry does not block a callback
whose persisted token proves its identity. The final callback removes the map entry only for its
own native task. Cancel captures the expected incarnation and optional live identifier, then
rechecks the current record/map before it calls `cancel()`. This prevents an old task callback or
delayed cancel request from mutating or cancelling a new task after the app has forgotten and reused
a terminal `TransferId`.

An ordinary launch must call `finish_launch_without_background_events`. After task reconciliation,
an absent active record without a commit intent becomes `Failed(Unavailable)`; a queued record with
no native task is resubmitted unless its cancel bit is set. A commit-intent record stays `Active`
when no matching task exists because the destination commit outcome is unknown. During
background-event startup, absent-task resolution waits for URLSession's finish-events callback. If
the OS never relaunches the app, no Rust status callback can run until a later app launch.

`start_download` creates a suspended task with a fresh incarnation tag, then atomically writes the
`Queued` record and token before registering/resuming the task. If that required write fails, it
cancels the still-suspended task and returns the write error without accepting the ID. It then
attempts an `Active` journal update. If that optional update fails, it still returns success because
the durable queued acceptance and matching task token allow reattachment. A later reconciliation
or terminal callback can update the record.

`cancel` writes a durable stop-request bit before it asks URLSession to cancel the matching task.
Success means the request was recorded, not that work has stopped. Status can remain `Queued` or
`Active` until URLSession reports a terminal result. A success or failure callback can win the
race; terminal records are not overwritten. A requested `NSURLErrorCancelled` becomes
`TransferStatus::Cancelled`; an OS cancellation with no app request becomes a failed status with
`ErrorKind::Cancelled` and the native code.

`didFinishDownloadingToURL` may detect invalid response metadata, an invalid destination, a
resource-limit error, or a B14 adoption error. B13 keeps those failures in memory under the
matching task incarnation and does not terminalize the record until the same task's later
`didCompleteWithError` callback. A queued replacement gets a fresh token and cannot inherit the old
incarnation's deferred failure. The task remains attached in the identity map through that final
callback, so `forget` and ID reuse cannot race an old task's remaining delegate messages. If the
process exits before `didCompleteWithError`, the deferred failure is lost; if reconciliation then
finds no task and no commit intent, it records `Failed(Unavailable)`. A reattached task may instead
complete with its native error. No recovered callback may claim `Succeeded` without this process's
live proof that B14 returned `Atomic`.

If a record has a pre-commit intent, `status` remains `Active`, `cancel` durably records its request
but does not terminalize it, and `forget` returns `NotTerminal`. A matching task-completion callback
or missing-task finalization terminalizes success from the persisted metadata only if the live
backend knows B14 returned an `Atomic` adoption result. On relaunch, an unresolved pre-commit intent starts without that in-memory proof, so a matching
`didCompleteWithError` callback or cancellation alone cannot terminalize it. A new matching
`didFinishDownloadingToURL` callback may re-adopt the downloaded file; only if that B14 call returns
`Atomic` does the live backend gain proof that permits later task completion or missing-task
finalization to persist `Succeeded`. Replay response-parse,
destination-validation, or adoption errors also preserve the unresolved intent as `Active`; without
a live `Atomic` receipt they cannot establish that the previous destination was untouched. A
terminal `cancel` is no-op success before any retry of a pending terminal journal write. `forget`
removes only a terminal record. It does not remove the destination file.

## Download install and limits

The `NSURLSessionDownloadDelegate` callback provides a temporary file URL that URLSession only
guarantees for the callback. B13 calls B14's `IosFiles::adopt_url_session_download` synchronously
before that callback returns. B14 copies with `io::copy` into a private same-directory staging file
and commits via atomic `renameat`; it does not allocate a payload-sized `Vec`, remove the temporary
source, or promise `fsync`/power-loss durability. A successful HTTP response, including a non-2xx
status, becomes `Succeeded` only after this atomic destination commit.

Before destination adoption, B13 size-preflights both the terminal record and a durable active
commit-intent record that contains the success status and response headers. If response metadata
cannot fit within 1 MiB, it records a pending `ResourceExhausted` failure in memory and does not
touch the destination; the terminal status is written only after the matching final task callback.
It then persists the intent and asks B14 to atomically install the file. URLSession's task-completion
callback, or missing-task finalization in the same live process, may persist `Succeeded` only after
B14 returned `Atomic`; the latter covers an early adoption callback followed by duplicate-group
cancellation. A first-attempt B14 adoption error proves the destination rename did not occur. B13
tries to clear the intent durably, then defers `Failed` until that task's final callback. If the
intent-clear write fails, B13 clears the intent only in live state and retains the incarnation-keyed
failure proof so the matching callback can still persist `Failed`; the disk journal may still hold
the old `Active` intent. If the process exits before terminal persistence, recovery keeps that
durable intent `Active`, because the no-rename proof existed only in memory. A replay error with an
existing intent preserves it because the previous process may already have replaced the destination.
A failure to persist the terminal result leaves the last durable journal record unchanged. After a
successful atomic adoption, that record is the active commit intent. After a first-attempt adoption
error whose intent clear persisted, it is `Active` without an intent; if no task reattaches, ordinary
or background missing-task reconciliation can resolve it to `Failed(Unavailable)` or `Cancelled`
from the durable cancel bit. If the intent-clear write failed, the durable record still has its
intent and remains `Active` after relaunch.

The file commit and terminal journal commit are separate atomic writes. If the terminal journal
write fails, B13 retains the terminal result in memory and retries on `status` or `forget`; a
`status` query returns the persistence error while retry still fails. If the process exits while an
unresolved commit intent remains durable, recovery cannot determine whether B14's rename happened:
B14 exposes no transaction receipt or rollback. B13 therefore keeps that transfer `Active` instead
of writing a false `Failed`, `Cancelled`, or `Succeeded`. This may strand an ID as `Active`
indefinitely; D10 guarantees the last durable status after relaunch but does not guarantee eventual
terminality. This preserves D10's rule that a failed/cancelled transfer leaves the prior destination
unchanged without claiming success before commit.

Request header bytes must be UTF-8 so Foundation can form `NSString` values. B13 uses
`NSMutableURLRequest.addValue_forHTTPHeaderField`; duplicate fields may be combined, and Foundation
or URLSession may normalize field spelling, order, and defaults. The backend does not promise
exact wire-level request-header fidelity.

`NSHTTPURLResponse.allHeaderFields` returns `NSDictionary`; duplicate response fields and source
order cannot be retained. B13 copies string values as UTF-8, converts `NSNumber` values with
`stringValue`, then sorts observed names for stable journal output. A value that cannot be
represented by D10's `ResponseHeader` fails before file adoption.

The backend leaves Foundation's cache, cookie, redirect, timeout, connectivity, and authentication
policies unchanged. Background tasks may continue while the app is suspended and the OS may relaunch
the app to deliver events. A user force-quit cancels background transfers and suppresses automatic
relaunch. No continuation guarantee applies after force-quit, OS resource policy, or power loss.

The effective API floor is iOS 10.0 from the existing `ios-files` surface; background URLSession is
documented from iOS 7.0 but does not lower B14's floor. See [file adoption](file-adoption.md) for
B14's SDK evidence and file-install contract.

The recorded device/simulator checks use Xcode 26.6 (build 17F113) and iPhoneOS/iPhoneSimulator
SDK 26.5, below the framework plan's Xcode 27.x baseline.

No progress stream, upload, body, custom retry, redirect override, shared task registry, or generic
connection pool is provided. Device/simulator compile checks do not prove OS relaunch or force-quit
behavior.
