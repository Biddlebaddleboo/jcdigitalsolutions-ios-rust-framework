use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use framework_background::{
    AppRefreshBackend, AppRefreshContext, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
    ExpirySignal, ExternalPowerRequirement, NetworkConnectivityRequirement, ProcessingTaskBackend,
    ProcessingTaskContext, ProcessingTaskId, ProcessingTaskOutcome, ProcessingTaskRequest,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode, Result};
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_background_tasks::{
    BGAppRefreshTaskRequest, BGProcessingTaskRequest, BGTask, BGTaskScheduler,
    BGTaskSchedulerErrorCode, BGTaskSchedulerErrorDomain,
};
use objc2_foundation::{NSError, NSString};
use std::ptr::NonNull;

use crate::lifecycle::{CompletionState, completion_succeeded};

/// A zero-state adapter for the shared iOS background-task scheduler
///
/// The host app owns launch order and must call each register method once before launch ends
#[derive(Clone, Copy, Debug, Default)]
pub struct IosBackgroundTasks;

impl IosBackgroundTasks {
    /// Creates a handle to the shared native scheduler
    pub const fn new() -> Self {
        Self
    }
}

impl AppRefreshBackend for IosBackgroundTasks {
    fn register_app_refresh<F>(&self, task_id: &AppRefreshTaskId, handler: F) -> Result<()>
    where
        F: for<'a> Fn(AppRefreshContext<'a>) -> AppRefreshOutcome + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let callback_id = task_id.clone();
        register_native_task(task_id.as_str(), move |native_task| {
            run_app_refresh(native_task, &callback_id, &handler);
        })
    }

    fn submit_app_refresh(&self, request: &AppRefreshRequest) -> Result<()> {
        let native_id = NSString::from_str(request.task_id().as_str());
        // SAFETY: the generated initializer accepts a valid NSString task ID and returns a
        // retained request object
        let native_request = unsafe {
            BGAppRefreshTaskRequest::initWithIdentifier(
                BGAppRefreshTaskRequest::alloc(),
                &native_id,
            )
        };
        // SAFETY: Apple's shared scheduler is a process singleton
        let scheduler = unsafe { BGTaskScheduler::sharedScheduler() };
        // SAFETY: the retained request remains live through the synchronous submit call
        unsafe {
            scheduler
                .submitTaskRequest_error(&native_request)
                .map_err(native_scheduler_error)
        }
    }

    fn cancel_app_refresh(&self, task_id: &AppRefreshTaskId) -> Result<()> {
        cancel_native_task(task_id.as_str())
    }
}

impl ProcessingTaskBackend for IosBackgroundTasks {
    fn register_processing_task<F>(&self, task_id: &ProcessingTaskId, handler: F) -> Result<()>
    where
        F: for<'a> Fn(ProcessingTaskContext<'a>) -> ProcessingTaskOutcome + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let callback_id = task_id.clone();
        register_native_task(task_id.as_str(), move |native_task| {
            run_processing_task(native_task, &callback_id, &handler);
        })
    }

    fn submit_processing_task(&self, request: &ProcessingTaskRequest) -> Result<()> {
        let native_id = NSString::from_str(request.task_id().as_str());
        // SAFETY: the generated initializer accepts a valid NSString task ID and returns a
        // retained request object
        let native_request = unsafe {
            BGProcessingTaskRequest::initWithIdentifier(
                BGProcessingTaskRequest::alloc(),
                &native_id,
            )
        };
        let requirements = request.requirements();
        // SAFETY: the retained request remains live and uniquely configured before submission
        unsafe {
            native_request.setRequiresNetworkConnectivity(
                requirements.network_connectivity() == NetworkConnectivityRequirement::Required,
            );
            native_request.setRequiresExternalPower(
                requirements.external_power() == ExternalPowerRequirement::Required,
            );
        }
        // SAFETY: Apple's shared scheduler is a process singleton
        let scheduler = unsafe { BGTaskScheduler::sharedScheduler() };
        // SAFETY: the retained request remains live through the synchronous submit call
        unsafe {
            scheduler
                .submitTaskRequest_error(&native_request)
                .map_err(native_scheduler_error)
        }
    }

    fn cancel_processing_task(&self, task_id: &ProcessingTaskId) -> Result<()> {
        cancel_native_task(task_id.as_str())
    }
}

fn register_native_task(
    task_id: &str,
    handler: impl Fn(NonNull<BGTask>) + Send + Sync + 'static,
) -> Result<()> {
    let native_id = NSString::from_str(task_id);
    let launch_handler = RcBlock::new(move |native_task: NonNull<BGTask>| handler(native_task));
    // SAFETY: Apple's shared scheduler is a process singleton. The launch block captures only a
    // Send + Sync handler. Each invocation confines its non-Send BGTask pointer to that call.
    let scheduler = unsafe { BGTaskScheduler::sharedScheduler() };
    // SAFETY: `None` selects Apple's default background queue; captured state is thread-safe.
    let registered = unsafe {
        scheduler.registerForTaskWithIdentifier_usingQueue_launchHandler(
            &native_id,
            None,
            &launch_handler,
        )
    };
    if registered {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::PermissionDenied))
    }
}

fn cancel_native_task(task_id: &str) -> Result<()> {
    let native_id = NSString::from_str(task_id);
    // SAFETY: Apple's shared scheduler is a process singleton
    let scheduler = unsafe { BGTaskScheduler::sharedScheduler() };
    // SAFETY: the ID remains live for the duration of this synchronous call
    unsafe { scheduler.cancelTaskRequestWithIdentifier(&native_id) };
    Ok(())
}

struct NativeExpiry(Arc<CompletionState>);

impl ExpirySignal for NativeExpiry {
    fn is_expired(&self) -> bool {
        self.0.is_expired()
    }
}

fn run_app_refresh<F>(native_task: NonNull<BGTask>, task_id: &AppRefreshTaskId, handler: &Arc<F>)
where
    F: for<'a> Fn(AppRefreshContext<'a>) -> AppRefreshOutcome + Send + Sync + 'static,
{
    run_native_task(native_task, |expiry| {
        handler(AppRefreshContext::new(task_id, expiry)) == AppRefreshOutcome::Succeeded
    });
}

fn run_processing_task<F>(
    native_task: NonNull<BGTask>,
    task_id: &ProcessingTaskId,
    handler: &Arc<F>,
) where
    F: for<'a> Fn(ProcessingTaskContext<'a>) -> ProcessingTaskOutcome + Send + Sync + 'static,
{
    run_native_task(native_task, |expiry| {
        handler(ProcessingTaskContext::new(task_id, expiry)) == ProcessingTaskOutcome::Succeeded
    });
}

fn run_native_task(native_task: NonNull<BGTask>, handler: impl FnOnce(&NativeExpiry) -> bool) {
    let state = Arc::new(CompletionState::new());
    let state_for_expiry = Arc::clone(&state);
    let callback_result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: BGTaskScheduler supplies a live BGTask object for this launch-handler call
        let task = unsafe { native_task.as_ref() };
        let expiry_handler = RcBlock::new(move || {
            let _ = state_for_expiry.expire();
        });
        // SAFETY: the native task lives for this callback. The expiry block captures only an
        // Arc<CompletionState>, so it is safe if Apple invokes or drops the copied block on another queue
        unsafe { task.setExpirationHandler(Some(&expiry_handler)) };

        let expiry = NativeExpiry(Arc::clone(&state));
        handler(&expiry)
    }));
    let success = completion_succeeded(&state, callback_result);

    // SAFETY: BGTaskScheduler supplies a live BGTask object for this launch-handler call. This
    // is the only native finish call; the atomic state makes expiry and closure return one-shot
    let task = unsafe { native_task.as_ref() };
    // SAFETY: the receiver is a live BGTask and this call follows one closure result
    unsafe { task.setTaskCompletedWithSuccess(success) };
}

fn native_scheduler_error(native_error: Retained<NSError>) -> Error {
    let code = native_error.code();
    // SAFETY: the generated binding exposes this immutable BackgroundTasks domain constant
    let scheduler_domain = unsafe { BGTaskSchedulerErrorDomain };
    let is_scheduler_error = native_error.domain().to_string() == scheduler_domain.to_string();
    let kind = if is_scheduler_error && code == BGTaskSchedulerErrorCode::Unavailable.0 {
        ErrorKind::Unavailable
    } else if is_scheduler_error && code == BGTaskSchedulerErrorCode::TooManyPendingTaskRequests.0 {
        ErrorKind::ResourceExhausted
    } else if is_scheduler_error && code == BGTaskSchedulerErrorCode::NotPermitted.0 {
        ErrorKind::PermissionDenied
    } else {
        ErrorKind::Platform
    };
    let error = Error::new(kind);
    match i32::try_from(code).ok().and_then(PlatformErrorCode::new) {
        Some(code) => error.with_platform_code(code),
        None => error,
    }
}
