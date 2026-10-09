use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::string::{String, ToString};
use std::sync::{Arc, Mutex, MutexGuard};
use std::vec::Vec;

use block2::{DynBlock, RcBlock};
use dispatch2::{DispatchQueue, MainThreadBound};
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use framework_files::{
    AppDirectory, AppPath, AtomicityRequirement, FileBackend, FileError, FileWriteMode,
    WriteAtomicity, WriteOptions,
};
use framework_network::{Header, HttpUrl, ResponseHeader, StatusCode};
use framework_transfer::{
    DownloadRequest, TransferBackend, TransferError, TransferId, TransferSnapshot, TransferStatus,
};
use ios_files::IosFiles;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, define_class, msg_send};
use objc2_foundation::{
    NSArray, NSDictionary, NSError, NSHTTPURLResponse, NSMutableURLRequest, NSNumber, NSObject,
    NSObjectProtocol, NSString, NSURL, NSURLSession, NSURLSessionConfiguration,
    NSURLSessionDelegate, NSURLSessionDownloadDelegate, NSURLSessionDownloadTask, NSURLSessionTask,
    NSURLSessionTaskDelegate, NSURLSessionTaskState, NSUUID,
};

use crate::record::{self, StoredHeader, StoredRecord};

const RECORD_ROOT: &str = "ios-transfer";
const TASK_PREFIX_V1: &str = "jcd-ios-transfer-v1:";
const TASK_PREFIX_V2: &str = "jcd-ios-transfer-v2:";

/// Why UIKit background-session event forwarding was not accepted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BackgroundEventError {
    /// The supplied identifier does not match this backend's session.
    SessionIdentifierMismatch,
    /// This backend already retains a UIKit completion handler.
    CompletionAlreadyPending,
    /// The app already classified this launch as ordinary rather than background-event startup.
    LaunchAlreadyClassified,
}

/// An app-owned background HTTP download backend.
///
/// Construct and retain this value on the main thread for the app lifetime. Recreate it with the
/// same unique session identifier after relaunch. This type is deliberately neither `Send` nor
/// `Sync`; synchronous file and journal operations may block the calling main thread.
pub struct IosTransferBackend {
    session_identifier: String,
    session: Retained<NSURLSession>,
    _delegate: Retained<TransferDelegate>,
    state: Arc<Mutex<BackendState>>,
    _main_thread_only: PhantomData<Rc<()>>,
}

impl Drop for IosTransferBackend {
    fn drop(&mut self) {
        let completion = { lock_state(&self.state).background_completion.take() };
        // The backend is !Send and callers must drop it on the main thread, so a pending
        // MainThreadBound block is released here on its owning thread rather than by a callback.
        drop(completion);
    }
}

impl IosTransferBackend {
    /// Opens a stable background URLSession and reloads this session's durable transfer records.
    ///
    /// `session_identifier` must be stable, unique to this backend in the app, and composed of
    /// ASCII letters, digits, `.`, `_`, and `-`, with an ASCII alphanumeric first byte. Call this
    /// on the main thread and keep the returned backend alive while its tasks or forwarded
    /// background events remain in flight.
    pub fn new(
        session_identifier: &str,
        _main_thread: MainThreadMarker,
    ) -> Result<Self, TransferError> {
        validate_session_identifier(session_identifier)?;
        let mut files = autoreleasepool(|_| IosFiles::new()).map_err(file_transfer_error)?;
        ensure_storage_directories(&mut files, session_identifier)?;
        let records = load_records(&mut files, session_identifier)?;
        let state = Arc::new(Mutex::new(BackendState {
            files,
            session_identifier: session_identifier.to_owned(),
            records,
            pending_writes: HashMap::new(),
            attached_tasks: HashMap::new(),
            pending_failures: HashMap::new(),
            adopted_ids: HashSet::new(),
            launch_mode: LaunchMode::Unclassified,
            reconcile_pending: true,
            events_finished: false,
            background_completion: None,
        }));
        let delegate = TransferDelegate::new(Arc::clone(&state));
        let (session, delegate) = autoreleasepool(|_| {
            let identifier = NSString::from_str(session_identifier);
            let configuration =
                NSURLSessionConfiguration::backgroundSessionConfigurationWithIdentifier(
                    &identifier,
                );
            let protocol_object = ProtocolObject::from_ref(&*delegate);
            // SAFETY: the delegate is retained in this backend and URLSession retains its
            // delegate; a nil operation queue asks Foundation to create its serial delegate queue.
            let session = unsafe {
                NSURLSession::sessionWithConfiguration_delegate_delegateQueue(
                    &configuration,
                    Some(protocol_object),
                    None,
                )
            };
            (session, delegate)
        });
        request_reconciliation(session.clone(), Arc::clone(&state));
        Ok(Self {
            session_identifier: session_identifier.to_owned(),
            session,
            _delegate: delegate,
            state,
            _main_thread_only: PhantomData,
        })
    }

    /// Retains UIKit's event completion handler until URLSession finishes delivering session
    /// events, then asynchronously invokes it once on the main dispatch queue.
    ///
    /// Call this from the app's `UIApplicationDelegate` callback on the main thread. The app owns
    /// the delegate and must forward only the matching identifier. The completion block is copied
    /// before this method returns. It remains pending until both URLSession's finish-events
    /// callback and initial task reconciliation have completed. Keep this backend alive until that
    /// handler is invoked; dropping it releases the retained block on main without invoking it.
    pub fn handle_background_events(
        &mut self,
        marker: MainThreadMarker,
        identifier: &str,
        completion_handler: &DynBlock<dyn Fn()>,
    ) -> Result<(), BackgroundEventError> {
        if identifier != self.session_identifier {
            return Err(BackgroundEventError::SessionIdentifierMismatch);
        }
        let mut state = lock_state(&self.state);
        if state.launch_mode == LaunchMode::Ordinary {
            return Err(BackgroundEventError::LaunchAlreadyClassified);
        }
        if state.background_completion.is_some() {
            return Err(BackgroundEventError::CompletionAlreadyPending);
        }
        // Preserve a finish callback that arrived before this handler was forwarded.
        state.launch_mode = LaunchMode::Background;
        state.background_completion = Some(MainThreadBound::new(completion_handler.copy(), marker));
        if state.events_finished && !state.reconcile_pending {
            finalize_missing_tasks(&mut state);
        }
        let ready = take_ready_completion(&mut state);
        drop(state);
        if let Some(completion) = ready {
            enqueue_completion(completion);
        }
        Ok(())
    }

    /// Reports ordinary app startup when UIKit did not provide a background URLSession event
    /// callback. This lets initial reattachment mark an active journal record failed if its
    /// native task no longer exists. Call once after app launch routing has ruled out the UIKit
    /// background-session callback.
    pub fn finish_launch_without_background_events(&mut self) -> Result<(), BackgroundEventError> {
        let mut state = lock_state(&self.state);
        if state.launch_mode != LaunchMode::Unclassified {
            return Err(BackgroundEventError::LaunchAlreadyClassified);
        }
        state.launch_mode = LaunchMode::Ordinary;
        if !state.reconcile_pending {
            finalize_missing_tasks(&mut state);
        }
        Ok(())
    }

    /// Retains a C-owned completion function and opaque context for the same main-queue event
    /// lifecycle as [`Self::handle_background_events`].
    ///
    /// # Safety
    ///
    /// `context` must remain valid until `callback` runs, and `callback` must not unwind. The
    /// callback runs exactly once on the main dispatch queue after URLSession finishes events and
    /// initial task reconciliation. The app must also retain this backend through that call.
    pub unsafe fn handle_background_events_c(
        &mut self,
        marker: MainThreadMarker,
        identifier: &str,
        callback: unsafe extern "C" fn(*mut std::ffi::c_void),
        context: *mut std::ffi::c_void,
    ) -> Result<(), BackgroundEventError> {
        if identifier != self.session_identifier {
            return Err(BackgroundEventError::SessionIdentifierMismatch);
        }
        let completion: RcBlock<dyn Fn()> = RcBlock::new(move || {
            // SAFETY: the caller promises that context stays valid until this single call.
            unsafe { callback(context) };
        });
        self.store_background_completion(marker, completion)
    }

    fn store_background_completion(
        &mut self,
        marker: MainThreadMarker,
        completion: RcBlock<dyn Fn()>,
    ) -> Result<(), BackgroundEventError> {
        let mut state = lock_state(&self.state);
        if state.launch_mode == LaunchMode::Ordinary {
            return Err(BackgroundEventError::LaunchAlreadyClassified);
        }
        if state.background_completion.is_some() {
            return Err(BackgroundEventError::CompletionAlreadyPending);
        }
        state.launch_mode = LaunchMode::Background;
        state.background_completion = Some(MainThreadBound::new(completion, marker));
        if state.events_finished && !state.reconcile_pending {
            finalize_missing_tasks(&mut state);
        }
        let ready = take_ready_completion(&mut state);
        drop(state);
        if let Some(completion) = ready {
            enqueue_completion(completion);
        }
        Ok(())
    }
}

impl TransferBackend for IosTransferBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn start_download(&mut self, request: DownloadRequest<'_>) -> Result<(), TransferError> {
        let id = request.id();
        let mut state = lock_state(&self.state);
        if state.records.contains_key(&id) {
            return Err(TransferError::DuplicateId);
        }
        if state.reconcile_pending {
            return Err(backend_error(ErrorKind::Unavailable, None));
        }
        let headers = request
            .headers()
            .iter()
            .map(|header| {
                if std::str::from_utf8(header.value()).is_err() {
                    return Err(backend_error(ErrorKind::Unsupported, None));
                }
                Ok(StoredHeader {
                    name: header.name().to_owned(),
                    value: header.value().to_vec(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let destination = request.destination();
        let record = StoredRecord {
            id,
            task_incarnation: Some(fresh_task_incarnation()),
            url: request.url().as_str().to_owned(),
            headers,
            directory: destination.directory(),
            destination: destination.relative().to_owned(),
            status: TransferStatus::Queued,
            cancel_requested: false,
            commit_intent: None,
        };
        let task = create_suspended_task(&self.session, &record)?;
        let task_identifier = task.taskIdentifier();
        if let Err(error) = persist_record(&mut state, &record) {
            autoreleasepool(|_| task.cancel());
            return Err(error);
        }
        state.attached_tasks.insert(id, task_identifier);
        autoreleasepool(|_| task.resume());
        let mut active = record;
        active.status = TransferStatus::Active;
        // The queued acceptance is already durable. If this optional status update fails, keep
        // that queued record and still return success: the tagged native task can be reattached.
        let _ = persist_record(&mut state, &active);
        Ok(())
    }

    fn status(&mut self, id: TransferId) -> Result<Option<TransferSnapshot>, TransferError> {
        let mut state = lock_state(&self.state);
        retry_pending_write(&mut state, id)?;
        Ok(state
            .records
            .get(&id)
            .map(|record| TransferSnapshot::new(record.id, record.status.clone())))
    }

    fn cancel(&mut self, id: TransferId) -> Result<(), TransferError> {
        {
            let mut state = lock_state(&self.state);
            let Some(current) = state.records.get(&id).cloned() else {
                return Err(TransferError::NotFound);
            };
            if current.status.is_terminal() {
                return Ok(());
            }
            retry_pending_write(&mut state, id)?;
            let Some(current) = state.records.get(&id).cloned() else {
                return Err(TransferError::NotFound);
            };
            if current.status.is_terminal() {
                return Ok(());
            }
            let mut requested = current;
            requested.cancel_requested = true;
            persist_record(&mut state, &requested)?;
        }
        let cancel_target = {
            let state = lock_state(&self.state);
            state.records.get(&id).and_then(|record| {
                record
                    .task_incarnation
                    .map(|incarnation| (incarnation, state.attached_tasks.get(&id).copied()))
            })
        };
        if let Some((incarnation, task_identifier)) = cancel_target {
            request_task_cancel(
                self.session.clone(),
                Arc::clone(&self.state),
                id,
                incarnation,
                task_identifier,
            );
        }
        Ok(())
    }

    fn forget(&mut self, id: TransferId) -> Result<(), TransferError> {
        let mut state = lock_state(&self.state);
        retry_pending_write(&mut state, id)?;
        let Some(record) = state.records.get(&id) else {
            return Err(TransferError::NotFound);
        };
        if !record.status.is_terminal() {
            return Err(TransferError::NotTerminal);
        }
        let path = record_path(&state.session_identifier, id);
        state
            .files
            .remove_file(app_path(&path)?)
            .map_err(file_transfer_error)?;
        state.records.remove(&id);
        state.attached_tasks.remove(&id);
        state.pending_failures.retain(|key, _| key.0 != id);
        state.adopted_ids.remove(&id);
        Ok(())
    }
}

struct BackendState {
    files: IosFiles,
    session_identifier: String,
    records: HashMap<TransferId, StoredRecord>,
    pending_writes: HashMap<TransferId, StoredRecord>,
    attached_tasks: HashMap<TransferId, usize>,
    pending_failures: HashMap<(TransferId, u128), Error>,
    adopted_ids: HashSet<TransferId>,
    reconcile_pending: bool,
    events_finished: bool,
    launch_mode: LaunchMode,
    background_completion: Option<MainThreadBound<RcBlock<dyn Fn()>>>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LaunchMode {
    Unclassified,
    Ordinary,
    Background,
}

struct TransferDelegateIvars {
    state: Arc<Mutex<BackendState>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = TransferDelegateIvars]
    struct TransferDelegate;

    // SAFETY: The class stores only an Arc to mutex-protected Rust state.
    unsafe impl NSObjectProtocol for TransferDelegate {}

    // SAFETY: These callbacks synchronize all mutable state, catch panics, and do not unwind
    // through Objective-C. The Foundation session uses its serial delegate queue.
    unsafe impl NSURLSessionDelegate for TransferDelegate {
        #[allow(non_snake_case)]
        #[unsafe(method(URLSessionDidFinishEventsForBackgroundURLSession:))]
        fn URLSessionDidFinishEventsForBackgroundURLSession(&self, _session: &NSURLSession) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                    return;
                };
                autoreleasepool(|_| finish_background_events(&self.ivars().state));
            }));
        }
    }

    // SAFETY: Task completion conversion and record updates use the synchronized state and catch
    // panics at the Objective-C boundary.
    unsafe impl NSURLSessionTaskDelegate for TransferDelegate {
        #[allow(non_snake_case)]
        #[unsafe(method(URLSession:task:didCompleteWithError:))]
        fn URLSession_task_didCompleteWithError(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionTask,
            error: Option<&NSError>,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                    return;
                };
                autoreleasepool(|_| complete_task(&self.ivars().state, task, error));
            }));
        }
    }

    // SAFETY: Download callbacks synchronously adopt the temporary file before returning and use
    // only the safe B14 file-adoption surface.
    unsafe impl NSURLSessionDownloadDelegate for TransferDelegate {
        #[allow(non_snake_case)]
        #[unsafe(method(URLSession:downloadTask:didFinishDownloadingToURL:))]
        fn URLSession_downloadTask_didFinishDownloadingToURL(
            &self,
            _session: &NSURLSession,
            task: &NSURLSessionDownloadTask,
            location: &NSURL,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                    return;
                };
                autoreleasepool(|_| finish_download(&self.ivars().state, task, location));
            }));
        }
    }
);

impl TransferDelegate {
    fn new(state: Arc<Mutex<BackendState>>) -> Retained<Self> {
        let allocated = Self::alloc().set_ivars(TransferDelegateIvars { state });
        // SAFETY: ivars are initialized before the NSObject initializer runs.
        unsafe { msg_send![super(allocated), init] }
    }
}

unsafe fn retain_callback_receiver(
    receiver: &TransferDelegate,
) -> Option<Retained<TransferDelegate>> {
    // SAFETY: objc2 supplies a live callback receiver; retaining it keeps ivars alive if the app
    // releases its backend reentrantly during a callback.
    unsafe { Retained::retain(receiver as *const TransferDelegate as *mut TransferDelegate) }
}

fn request_reconciliation(session: Retained<NSURLSession>, state: Arc<Mutex<BackendState>>) {
    let callback_session = session.clone();
    let completion = RcBlock::new(move |tasks: std::ptr::NonNull<NSArray<NSURLSessionTask>>| {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: Foundation invokes the completion block with a live NSArray for this
                // callback; the array is used only before the callback returns.
                reconcile_tasks(&state, &callback_session, unsafe { tasks.as_ref() });
            });
        }));
    });
    // SAFETY: the block captures only `Arc<Mutex<BackendState>>` and `Retained<NSURLSession>`,
    // both Send + Sync; its callback is safe on Foundation's session delegate queue. The local
    // RcBlock remains alive through the call and Foundation copies completion blocks it retains.
    unsafe { session.getAllTasksWithCompletionHandler(&completion) };
}

fn request_task_cancel(
    session: Retained<NSURLSession>,
    state: Arc<Mutex<BackendState>>,
    id: TransferId,
    incarnation: u128,
    attached_identifier: Option<usize>,
) {
    let completion = RcBlock::new(move |tasks: std::ptr::NonNull<NSArray<NSURLSessionTask>>| {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: Foundation supplies a live task array for the callback duration.
                let tasks = unsafe { tasks.as_ref() };
                let state = lock_state(&state);
                if state
                    .records
                    .get(&id)
                    .and_then(|record| record.task_incarnation)
                    != Some(incarnation)
                {
                    return;
                }
                if state
                    .records
                    .get(&id)
                    .is_some_and(|record| record.status.is_terminal())
                {
                    return;
                }
                if attached_identifier.is_some()
                    && state.attached_tasks.get(&id) != attached_identifier.as_ref()
                {
                    return;
                }
                for index in 0..tasks.count() {
                    let task = tasks.objectAtIndex(index);
                    if task_matches_identity(&task, id, incarnation)
                        && task_matches_record(&state, id, &task)
                        && attached_identifier.is_none_or(|value| value == task.taskIdentifier())
                    {
                        task.cancel();
                        break;
                    }
                }
            });
        }));
    });
    // SAFETY: this completion captures only thread-safe state and scalar identifiers. Foundation
    // copies its asynchronous completion block before returning.
    unsafe { session.getAllTasksWithCompletionHandler(&completion) };
}

fn reconcile_tasks(
    state: &Arc<Mutex<BackendState>>,
    session: &NSURLSession,
    tasks: &NSArray<NSURLSessionTask>,
) {
    let mut state = lock_state(state);
    let mut tagged: HashMap<TransferId, Vec<Retained<NSURLSessionTask>>> = HashMap::new();
    for index in 0..tasks.count() {
        let task = tasks.objectAtIndex(index);
        let Some((id, _)) = task_transfer_identity(&task) else {
            // This identifier is dedicated to this backend, so an untagged task is an orphan
            // created in the crash window before taskDescription was assigned. New tasks are
            // suspended until after the description is set.
            task.cancel();
            continue;
        };
        tagged.entry(id).or_default().push(task);
    }
    let mut found = HashSet::new();
    let mut attached_tasks = HashMap::new();
    for (id, mut matching_tasks) in tagged {
        if matching_tasks.len() != 1 {
            // If multiple native tasks carry one TransferId, choosing one could attach an old
            // incarnation to a reused ID. Only a callback whose token matches the journal record
            // can act before this conservative group cancellation.
            for task in matching_tasks {
                task.setTaskDescription(None);
                task.cancel();
            }
            continue;
        }
        let Some(task) = matching_tasks.pop() else {
            continue;
        };
        let Some(mut record) = state.records.get(&id).cloned() else {
            task.setTaskDescription(None);
            task.cancel();
            continue;
        };
        let task_identifier = task.taskIdentifier();
        let tagged_incarnation = task_transfer_identity(&task).and_then(|(_, value)| value);
        if record.task_incarnation.is_none() || record.task_incarnation != tagged_incarnation {
            // JCDITF01 records and old task tags lack an incarnation token. Do not attach them:
            // a reused TransferId cannot safely distinguish an old native task from its current
            // record until queued work is replaced with a tokenized task.
            task.setTaskDescription(None);
            task.cancel();
            continue;
        }
        if record.status.is_terminal() {
            task.setTaskDescription(None);
            task.cancel();
            continue;
        }
        found.insert(id);
        attached_tasks.insert(id, task_identifier);
        if record.cancel_requested {
            task.cancel();
        } else {
            if record.status == TransferStatus::Queued {
                record.status = TransferStatus::Active;
                let _ = persist_record(&mut state, &record);
            }
            if task.state() == NSURLSessionTaskState::Suspended {
                task.resume();
            }
        }
    }
    state.attached_tasks = attached_tasks;
    let queued: Vec<StoredRecord> = state
        .records
        .values()
        .filter(|record| {
            record.status == TransferStatus::Queued
                && !record.cancel_requested
                && !found.contains(&record.id)
        })
        .cloned()
        .collect();
    for mut record in queued {
        record.task_incarnation = Some(fresh_task_incarnation());
        match create_suspended_task(session, &record) {
            Ok(task) => {
                let task_identifier = task.taskIdentifier();
                let Some(incarnation) = record.task_incarnation else {
                    task.cancel();
                    continue;
                };
                if persist_record(&mut state, &record).is_err() {
                    task.cancel();
                    continue;
                }
                state
                    .pending_failures
                    .retain(|key, _| key.0 != record.id || key.1 == incarnation);
                state.attached_tasks.insert(record.id, task_identifier);
                task.resume();
                record.status = TransferStatus::Active;
                let _ = persist_record(&mut state, &record);
            }
            Err(error) => {
                record.status = TransferStatus::Failed(match error {
                    TransferError::Backend(error) => error,
                    _ => Error::new(ErrorKind::Internal),
                });
                let _ = persist_record(&mut state, &record);
            }
        }
    }
    state.reconcile_pending = false;
    if state.launch_mode == LaunchMode::Ordinary
        || (state.launch_mode == LaunchMode::Background && state.events_finished)
    {
        finalize_missing_tasks(&mut state);
    }
    let completion = take_ready_completion(&mut state);
    drop(state);
    if let Some(completion) = completion {
        enqueue_completion(completion);
    }
}

fn finish_background_events(state: &Arc<Mutex<BackendState>>) {
    let completion = {
        let mut state = lock_state(state);
        state.events_finished = true;
        if !state.reconcile_pending && state.launch_mode == LaunchMode::Background {
            finalize_missing_tasks(&mut state);
        }
        take_ready_completion(&mut state)
    };
    if let Some(completion) = completion {
        enqueue_completion(completion);
    }
}

fn finalize_missing_tasks(state: &mut BackendState) {
    let missing: Vec<StoredRecord> = state
        .records
        .values()
        .filter(|record| {
            !record.status.is_terminal() && !state.attached_tasks.contains_key(&record.id)
        })
        .cloned()
        .collect();
    for mut record in missing {
        if let Some(success) = record.commit_intent.clone() {
            if state.adopted_ids.remove(&record.id) {
                record.status = TransferStatus::Succeeded {
                    status: success.status,
                    headers: success.headers,
                };
                record.commit_intent = None;
                let _ = persist_record(state, &record);
            }
            continue;
        }
        // A durable commit intent means adoption may have happened immediately before process
        // exit. B14 provides no receipt to distinguish that case from a crash just before rename;
        // recovered intents therefore remain Active rather than become false failure/cancellation.
        record.status = if let Some(error) = record
            .task_incarnation
            .and_then(|incarnation| state.pending_failures.remove(&(record.id, incarnation)))
        {
            TransferStatus::Failed(error)
        } else if record.cancel_requested {
            TransferStatus::Cancelled
        } else {
            TransferStatus::Failed(Error::new(ErrorKind::Unavailable))
        };
        let _ = persist_record(state, &record);
    }
}

fn take_ready_completion(state: &mut BackendState) -> Option<MainThreadBound<RcBlock<dyn Fn()>>> {
    if state.launch_mode == LaunchMode::Background
        && state.events_finished
        && !state.reconcile_pending
    {
        let completion = state.background_completion.take();
        if completion.is_some() {
            state.events_finished = false;
        }
        completion
    } else {
        None
    }
}

fn enqueue_completion(completion: MainThreadBound<RcBlock<dyn Fn()>>) {
    DispatchQueue::main().exec_async(move || {
        // SAFETY: this closure is submitted to DispatchQueue::main, so it runs on the main thread.
        let marker = unsafe { MainThreadMarker::new_unchecked() };
        let block = completion.into_inner(marker);
        let _ = catch_unwind(AssertUnwindSafe(|| block.call(())));
        // `block` is dropped here on the main thread after its single invocation.
    });
}

fn complete_task(
    state: &Arc<Mutex<BackendState>>,
    task: &NSURLSessionTask,
    error: Option<&NSError>,
) {
    let Some((id, Some(incarnation))) = task_transfer_identity(task) else {
        return;
    };
    let mut state = lock_state(state);
    if !task_matches_record(&state, id, task) {
        return;
    }
    if state
        .attached_tasks
        .get(&id)
        .is_some_and(|attached| *attached == task.taskIdentifier())
    {
        state.attached_tasks.remove(&id);
    }
    let Some(mut record) = state.records.get(&id).cloned() else {
        return;
    };
    if record.status.is_terminal() {
        state.pending_failures.remove(&(id, incarnation));
        return;
    }
    if let Some(success) = record.commit_intent.clone() {
        state.pending_failures.remove(&(id, incarnation));
        if state.adopted_ids.remove(&id) {
            record.status = TransferStatus::Succeeded {
                status: success.status,
                headers: success.headers,
            };
            record.commit_intent = None;
            let _ = persist_record(&mut state, &record);
        }
        return;
    }
    if let Some(pending_failure) = state.pending_failures.remove(&(id, incarnation)) {
        record.status = TransferStatus::Failed(pending_failure);
    } else if let Some(error) = error {
        record.status =
            if error.code() == objc2_foundation::NSURLErrorCancelled && record.cancel_requested {
                TransferStatus::Cancelled
            } else if error.code() == objc2_foundation::NSURLErrorCancelled {
                TransferStatus::Failed(platform_error(
                    ErrorKind::Cancelled,
                    i32::try_from(error.code()).unwrap_or(0),
                ))
            } else {
                TransferStatus::Failed(platform_error(
                    ErrorKind::Platform,
                    i32::try_from(error.code()).unwrap_or(0),
                ))
            };
    } else if !matches!(record.status, TransferStatus::Succeeded { .. }) {
        record.status = TransferStatus::Failed(Error::new(ErrorKind::Internal));
    }
    record.commit_intent = None;
    let _ = persist_record(&mut state, &record);
}

fn finish_download(
    state: &Arc<Mutex<BackendState>>,
    task: &NSURLSessionDownloadTask,
    location: &NSURL,
) {
    let Some(id) = task_transfer_id(task) else {
        return;
    };
    let response = task.response();
    let Some(response) =
        response.and_then(|response| response.downcast::<NSHTTPURLResponse>().ok())
    else {
        defer_failure(state, task, Error::new(ErrorKind::InvalidInput));
        return;
    };
    let Some(status) = u16::try_from(response.statusCode())
        .ok()
        .and_then(StatusCode::new)
    else {
        defer_failure(state, task, Error::new(ErrorKind::InvalidInput));
        return;
    };
    let Some(headers) = response_headers(&response) else {
        defer_failure(state, task, Error::new(ErrorKind::InvalidInput));
        return;
    };
    let mut state = lock_state(state);
    if !task_matches_record(&state, id, task) {
        return;
    }
    let Some(record) = state.records.get(&id).cloned() else {
        return;
    };
    if record.status.is_terminal() {
        return;
    }
    let had_commit_intent = record.commit_intent.is_some();
    let Ok(destination) = AppPath::new(record.directory, &record.destination) else {
        if had_commit_intent {
            return;
        }
        defer_failure_locked(&mut state, id, task, Error::new(ErrorKind::InvalidInput));
        return;
    };
    let mut terminal = record.clone();
    terminal.status = TransferStatus::Succeeded {
        status,
        headers: headers.clone(),
    };
    terminal.commit_intent = None;
    if record::encode(&terminal).is_none() {
        if had_commit_intent {
            return;
        }
        defer_failure_locked(
            &mut state,
            id,
            task,
            Error::new(ErrorKind::ResourceExhausted),
        );
        return;
    }
    let mut intent = record.clone();
    intent.status = TransferStatus::Active;
    intent.commit_intent = Some(record::CommitIntent {
        status,
        headers: headers.clone(),
    });
    if record::encode(&intent).is_none() {
        if had_commit_intent {
            return;
        }
        defer_failure_locked(
            &mut state,
            id,
            task,
            Error::new(ErrorKind::ResourceExhausted),
        );
        return;
    }
    if let Err(error) = persist_record(&mut state, &intent) {
        if had_commit_intent {
            return;
        }
        defer_failure_locked(
            &mut state,
            id,
            task,
            match error {
                TransferError::Backend(error) => error,
                _ => Error::new(ErrorKind::Internal),
            },
        );
        return;
    }
    match state
        .files
        .adopt_url_session_download(location, destination)
    {
        Ok(outcome) if outcome.atomicity() == WriteAtomicity::Atomic => {
            state.adopted_ids.insert(id);
        }
        // A non-atomic result is outside B14's current contract. Keep the intent unresolved
        // rather than claim failure after an operation whose destination effect is unknown.
        Ok(_) => {}
        Err(error) if !had_commit_intent => {
            // B14 guarantees that an adoption error leaves the prior destination unchanged. Clear
            // the intent durably when possible, but defer the terminal failure until
            // didCompleteWithError so this task incarnation remains attached through its final
            // native callback. If clearing the journal intent fails, retain the same no-rename
            // proof in live state: the matching callback can still persist Failed, while a process
            // exit leaves the durable intent unresolved rather than inventing an outcome.
            intent.commit_intent = None;
            if persist_record(&mut state, &intent).is_err() {
                state.records.insert(id, intent);
            }
            defer_failure_locked(&mut state, id, task, file_error_value(error));
        }
        Err(_) => {}
    }
}

fn response_headers(response: &NSHTTPURLResponse) -> Option<Vec<ResponseHeader>> {
    let fields: Retained<NSDictionary> = response.allHeaderFields();
    let keys = fields.allKeys();
    let mut headers = Vec::with_capacity(keys.count());
    for index in 0..keys.count() {
        let key: Retained<AnyObject> = keys.objectAtIndex(index);
        let key = key.downcast_ref::<NSString>()?;
        let value: Retained<AnyObject> = fields.objectForKey(key)?;
        let value = if let Some(value) = value.downcast_ref::<NSString>() {
            value.to_string()
        } else {
            value.downcast_ref::<NSNumber>()?.stringValue().to_string()
        };
        headers.push(ResponseHeader::new(key.to_string(), value.into_bytes()).ok()?);
    }
    headers.sort_by(|left, right| left.name().cmp(right.name()));
    Some(headers)
}

fn defer_failure(state: &Arc<Mutex<BackendState>>, task: &NSURLSessionDownloadTask, error: Error) {
    let Some(id) = task_transfer_id(task) else {
        return;
    };
    let mut state = lock_state(state);
    defer_failure_locked(&mut state, id, task, error);
}

fn defer_failure_locked(
    state: &mut BackendState,
    id: TransferId,
    task: &NSURLSessionDownloadTask,
    error: Error,
) {
    if !task_matches_record(state, id, task) {
        return;
    }
    let Some(record) = state.records.get(&id) else {
        return;
    };
    if record.commit_intent.is_some() {
        return;
    }
    if !record.status.is_terminal() {
        let Some((_, Some(incarnation))) = task_transfer_identity(task) else {
            return;
        };
        state.pending_failures.insert((id, incarnation), error);
    }
}

fn task_transfer_id(task: &NSURLSessionTask) -> Option<TransferId> {
    task_transfer_identity(task).map(|(id, _)| id)
}

fn task_transfer_identity(task: &NSURLSessionTask) -> Option<(TransferId, Option<u128>)> {
    let description = task.taskDescription()?;
    let value = description.to_string();
    if let Some(id) = value.strip_prefix(TASK_PREFIX_V1) {
        if id.len() != 32 {
            return None;
        }
        return u128::from_str_radix(id, 16)
            .ok()
            .and_then(TransferId::new)
            .map(|id| (id, None));
    }
    let identity = value.strip_prefix(TASK_PREFIX_V2)?;
    let (id, incarnation) = identity.split_once(':')?;
    if id.len() != 32 || incarnation.len() != 32 {
        return None;
    }
    Some((
        TransferId::new(u128::from_str_radix(id, 16).ok()?)?,
        Some(u128::from_str_radix(incarnation, 16).ok()?),
    ))
}

fn task_matches_identity(task: &NSURLSessionTask, id: TransferId, incarnation: u128) -> bool {
    task_transfer_identity(task) == Some((id, Some(incarnation)))
}

fn task_matches_record(state: &BackendState, id: TransferId, task: &NSURLSessionTask) -> bool {
    let Some(record) = state.records.get(&id) else {
        return false;
    };
    let Some(incarnation) = record.task_incarnation else {
        return false;
    };
    task_matches_identity(task, id, incarnation)
        && state
            .attached_tasks
            .get(&id)
            .is_none_or(|attached| *attached == task.taskIdentifier())
}

fn task_description(id: TransferId, incarnation: u128) -> String {
    format!("{TASK_PREFIX_V2}{:032x}:{:032x}", id.get(), incarnation)
}

fn fresh_task_incarnation() -> u128 {
    loop {
        let value = autoreleasepool(|_| NSUUID::UUID().UUIDString().to_string());
        let mut hex = String::with_capacity(32);
        for byte in value.bytes().filter(u8::is_ascii_hexdigit) {
            hex.push(char::from(byte));
        }
        if hex.len() == 32 {
            if let Ok(incarnation) = u128::from_str_radix(&hex, 16) {
                if incarnation != 0 {
                    return incarnation;
                }
            }
        }
    }
}

fn create_suspended_task(
    session: &NSURLSession,
    record: &StoredRecord,
) -> Result<Retained<NSURLSessionDownloadTask>, TransferError> {
    let incarnation = record
        .task_incarnation
        .ok_or_else(|| backend_error(ErrorKind::InvalidInput, None))?;
    let request = autoreleasepool(|_| native_request(record))?;
    Ok(autoreleasepool(|_| {
        let task = session.downloadTaskWithRequest(&request);
        let description = NSString::from_str(&task_description(record.id, incarnation));
        task.setTaskDescription(Some(&description));
        task
    }))
}

fn native_request(record: &StoredRecord) -> Result<Retained<NSMutableURLRequest>, TransferError> {
    let url_string = NSString::from_str(&record.url);
    let url = NSURL::URLWithString(&url_string)
        .ok_or_else(|| backend_error(ErrorKind::InvalidInput, None))?;
    let request = NSMutableURLRequest::requestWithURL(&url);
    let method = NSString::from_str("GET");
    request.setHTTPMethod(&method);
    for header in &record.headers {
        let value = std::str::from_utf8(&header.value)
            .map_err(|_| backend_error(ErrorKind::Unsupported, None))?;
        let name = NSString::from_str(&header.name);
        let value = NSString::from_str(value);
        // NSURLRequest combines duplicate fields and may normalize field spelling or order.
        request.addValue_forHTTPHeaderField(&value, &name);
    }
    Ok(request)
}

fn ensure_storage_directories(
    files: &mut IosFiles,
    session_identifier: &str,
) -> Result<(), TransferError> {
    for relative in [
        RECORD_ROOT.to_owned(),
        format!("{RECORD_ROOT}/{session_identifier}"),
    ] {
        let path = app_path(&relative)?;
        if !files.exists(path).map_err(file_transfer_error)? {
            files.create_directory(path).map_err(file_transfer_error)?;
        }
    }
    Ok(())
}

fn load_records(
    files: &mut IosFiles,
    session_identifier: &str,
) -> Result<HashMap<TransferId, StoredRecord>, TransferError> {
    let directory_name = format!("{RECORD_ROOT}/{session_identifier}");
    let directory = app_path(&directory_name)?;
    let entries = files
        .read_directory(directory)
        .map_err(file_transfer_error)?;
    let mut records = HashMap::new();
    for entry in entries {
        if entry.kind() != framework_files::FileKind::File || !entry.name().ends_with(".record") {
            continue;
        }
        let path = record_path(session_identifier, parse_record_id(entry.name())?);
        let bytes = files.read(app_path(&path)?).map_err(file_transfer_error)?;
        let record =
            record::decode(&bytes).ok_or_else(|| backend_error(ErrorKind::Internal, None))?;
        if path != record_path(session_identifier, record.id)
            || HttpUrl::new(&record.url).is_err()
            || AppPath::new(record.directory, &record.destination).is_err()
            || record
                .headers
                .iter()
                .any(|header| Header::new(&header.name, &header.value).is_err())
        {
            return Err(backend_error(ErrorKind::Internal, None));
        }
        if records.insert(record.id, record).is_some() {
            return Err(backend_error(ErrorKind::Internal, None));
        }
    }
    Ok(records)
}

fn persist_record(state: &mut BackendState, record: &StoredRecord) -> Result<(), TransferError> {
    let Some(bytes) = record::encode(record) else {
        let error = backend_error(ErrorKind::ResourceExhausted, None);
        if record.status.is_terminal() {
            state.records.insert(record.id, record.clone());
            state.pending_writes.insert(record.id, record.clone());
        }
        return Err(error);
    };
    let path = record_path(&state.session_identifier, record.id);
    let outcome = match state.files.write(
        app_path(&path)?,
        &bytes,
        WriteOptions::new(
            FileWriteMode::CreateOrReplace,
            AtomicityRequirement::RequireAtomic,
        ),
    ) {
        Ok(outcome) => outcome,
        Err(error) => {
            let error = file_transfer_error(error);
            if record.status.is_terminal() {
                state.records.insert(record.id, record.clone());
                state.pending_writes.insert(record.id, record.clone());
            }
            return Err(error);
        }
    };
    if outcome.atomicity() != WriteAtomicity::Atomic {
        if record.status.is_terminal() {
            state.records.insert(record.id, record.clone());
            state.pending_writes.insert(record.id, record.clone());
        }
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    state.records.insert(record.id, record.clone());
    state.pending_writes.remove(&record.id);
    if record.status.is_terminal() {
        state.pending_failures.retain(|key, _| key.0 != record.id);
    }
    Ok(())
}

fn retry_pending_write(state: &mut BackendState, id: TransferId) -> Result<(), TransferError> {
    let Some(record) = state.pending_writes.get(&id).cloned() else {
        return Ok(());
    };
    persist_record(state, &record)
}

fn app_path(path: &str) -> Result<AppPath<'_>, TransferError> {
    AppPath::new(AppDirectory::ApplicationSupport, path)
        .map_err(|error| backend_error(error.kind(), error.platform_code().map(|code| code.get())))
}

fn record_path(session_identifier: &str, id: TransferId) -> String {
    format!(
        "{RECORD_ROOT}/{session_identifier}/{:032x}.record",
        id.get()
    )
}

fn parse_record_id(name: &str) -> Result<TransferId, TransferError> {
    let Some(hex) = name.strip_suffix(".record") else {
        return Err(backend_error(ErrorKind::Internal, None));
    };
    if hex.len() != 32 {
        return Err(backend_error(ErrorKind::Internal, None));
    }
    u128::from_str_radix(hex, 16)
        .ok()
        .and_then(TransferId::new)
        .ok_or_else(|| backend_error(ErrorKind::Internal, None))
}

fn validate_session_identifier(value: &str) -> Result<(), TransferError> {
    let valid = (1..=128).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(backend_error(ErrorKind::InvalidInput, None))
    }
}

fn file_transfer_error(error: FileError) -> TransferError {
    TransferError::Backend(platform_error(
        error.kind(),
        error.platform_code().map(|code| code.get()).unwrap_or(0),
    ))
}

fn file_error_value(error: FileError) -> Error {
    platform_error(
        error.kind(),
        error.platform_code().map(|code| code.get()).unwrap_or(0),
    )
}

fn backend_error(kind: ErrorKind, code: Option<i32>) -> TransferError {
    TransferError::Backend(platform_error(kind, code.unwrap_or(0)))
}

fn platform_error(kind: ErrorKind, code: i32) -> Error {
    PlatformErrorCode::new(code).map_or_else(
        || Error::new(kind),
        |code| Error::new(kind).with_platform_code(code),
    )
}

fn lock_state(state: &Mutex<BackendState>) -> MutexGuard<'_, BackendState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
