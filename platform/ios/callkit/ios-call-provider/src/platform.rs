use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

use block2::RcBlock;
use dispatch2::{DispatchObject, DispatchQueue, DispatchRetained, MainThreadBound};
use objc2::MainThreadMarker;
use objc2::MainThreadOnly;
use objc2::rc::{Retained, Weak, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, available, define_class, msg_send};
use objc2_call_kit::{
    CXAction, CXAnswerCallAction, CXCallAction, CXCallEndedReason, CXCallUpdate, CXEndCallAction,
    CXHandle, CXHandleType, CXProvider, CXProviderConfiguration, CXProviderDelegate,
};
use objc2_foundation::{NSError, NSNumber, NSObject, NSObjectProtocol, NSSet, NSString, NSUUID};

use crate::registry::{BeginReportError, CallRegistry, EndTransition, OnceSlot, ReportTransition};
use crate::{
    ActionDecision, CallEndReason, CallHandleKind, CallId, CallProviderConfig, CallProviderError,
    CallProviderEvent, EndCallOutcome, IncomingCall,
};

type EventHandler = Box<dyn FnMut(CallProviderEvent) -> ActionDecision + 'static>;
type ReportCompletion = Box<dyn FnOnce(Result<(), CallProviderError>) + 'static>;

struct ProviderShared {
    calls: RefCell<CallRegistry>,
    handler: RefCell<EventHandler>,
}

struct ReportCallbackContext {
    shared: Rc<ProviderShared>,
    weak_provider: Weak<CXProvider>,
    call_id: CallId,
    generation: u64,
    uuid: Retained<NSUUID>,
    completion: Rc<RefCell<Option<ReportCompletion>>>,
}

struct ProviderDelegateIvars {
    shared: Rc<ProviderShared>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ProviderDelegateIvars]
    struct ProviderDelegate;

    // SAFETY: NSObjectProtocol adds no methods and this class is confined to its main thread.
    unsafe impl NSObjectProtocol for ProviderDelegate {}

    // SAFETY: CallKit's delegate queue is the main queue, matching the class thread kind and
    // RefCell-owned state. Every callback contains Rust panics and never retains a CXAction.
    #[allow(non_snake_case)]
    unsafe impl CXProviderDelegate for ProviderDelegate {
        #[unsafe(method(providerDidReset:))]
        unsafe fn providerDidReset(&self, _provider: &CXProvider) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.ivars().shared.calls.borrow_mut().reset();
                dispatch_event(&self.ivars().shared, CallProviderEvent::ProviderReset);
            }));
        }

        #[unsafe(method(provider:performAnswerCallAction:))]
        unsafe fn provider_performAnswerCallAction(
            &self,
            _provider: &CXProvider,
            action: &CXAnswerCallAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    complete_call_action(&self.ivars().shared, action, ActionKind::Answer);
                });
            }));
        }

        #[unsafe(method(provider:performEndCallAction:))]
        unsafe fn provider_performEndCallAction(
            &self,
            _provider: &CXProvider,
            action: &CXEndCallAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    complete_call_action(&self.ivars().shared, action, ActionKind::End);
                });
            }));
        }

        #[unsafe(method(provider:timedOutPerformingAction:))]
        unsafe fn provider_timedOutPerformingAction(
            &self,
            _provider: &CXProvider,
            _action: &CXAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                dispatch_event(&self.ivars().shared, CallProviderEvent::ActionTimedOut);
            }));
        }
    }
);

impl ProviderDelegate {
    fn new(main_thread: MainThreadMarker, shared: Rc<ProviderShared>) -> Retained<Self> {
        let allocated = Self::alloc(main_thread).set_ivars(ProviderDelegateIvars { shared });
        // SAFETY: The Objective-C subclass has no custom initializer and all ivars are set.
        unsafe { msg_send![super(allocated), init] }
    }
}

#[derive(Clone, Copy)]
enum ActionKind {
    Answer,
    End,
}

fn dispatch_event(shared: &ProviderShared, event: CallProviderEvent) -> ActionDecision {
    catch_unwind(AssertUnwindSafe(|| (shared.handler.borrow_mut())(event)))
        .unwrap_or(ActionDecision::Fail)
}

fn complete_call_action(shared: &ProviderShared, action: &CXCallAction, kind: ActionKind) {
    let id = unsafe { action.callUUID() };
    let Some(call_id) = call_id_from_native(&id) else {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    };
    if !shared.calls.borrow().is_active(call_id) {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    }

    let event = match kind {
        ActionKind::Answer => CallProviderEvent::AnswerRequested { call_id },
        ActionKind::End => CallProviderEvent::EndRequested { call_id },
    };
    let decision = dispatch_event(shared, event);

    // CallKit may have timed out the action while user code ran. Timed-out actions are already
    // failed by the system and must not be completed again.
    if unsafe { action.isComplete() } {
        return;
    }
    match decision {
        ActionDecision::Fulfill => {
            if matches!(kind, ActionKind::End) {
                shared.calls.borrow_mut().finish_end_action(call_id);
            }
            unsafe { action.fulfill() };
        }
        ActionDecision::Fail => unsafe { action.fail() },
    }
}

/// One explicitly owned CallKit provider.
///
/// Create and use this value on the main thread. Keep it alive for the complete provider
/// lifecycle, then call [`invalidate`](Self::invalidate) before release. The provider delegate is
/// retained by this value because CallKit stores its delegate weakly. The value is not `Send` or
/// `Sync`.
pub struct CallProvider {
    provider: Option<Retained<CXProvider>>,
    delegate: Option<Retained<ProviderDelegate>>,
    delegate_queue: Option<DispatchRetained<DispatchQueue>>,
    shared: Rc<ProviderShared>,
    config: CallProviderConfig,
    _main_thread: MainThreadMarker,
}

impl CallProvider {
    /// Create a provider on the main thread with an immediate answer/end action handler.
    ///
    /// The handler runs synchronously on the CallKit delegate queue, which is the main queue. It
    /// must return promptly and must not wait for network work. For `ProviderReset` and
    /// `ActionTimedOut` events, its return value is ignored. Panics become failed actions.
    pub fn new<F>(
        main_thread: MainThreadMarker,
        config: CallProviderConfig,
        handler: F,
    ) -> Result<Self, CallProviderError>
    where
        F: FnMut(CallProviderEvent) -> ActionDecision + 'static,
    {
        config.validate()?;
        let native_config = native_provider_config(&config)?;
        let provider =
            unsafe { CXProvider::initWithConfiguration(CXProvider::alloc(), &native_config) };
        let shared = Rc::new(ProviderShared {
            calls: RefCell::new(CallRegistry::default()),
            handler: RefCell::new(Box::new(handler)),
        });
        let delegate = ProviderDelegate::new(main_thread, Rc::clone(&shared));
        let delegate_object = ProtocolObject::from_ref(&*delegate);
        let delegate_queue = DispatchQueue::main().retain();
        // SAFETY: The provider and delegate are created on the main thread. The retained main
        // queue serializes delegate callbacks with MainThreadOnly state.
        unsafe { provider.setDelegate_queue(Some(delegate_object), Some(&delegate_queue)) };

        Ok(Self {
            provider: Some(provider),
            delegate: Some(delegate),
            delegate_queue: Some(delegate_queue),
            shared,
            config,
            _main_thread: main_thread,
        })
    }

    /// Report an incoming call. The completion runs once asynchronously on the main queue.
    ///
    /// CallKit can reject a call due to system policy; that returns
    /// [`CallProviderError::SystemRejected`] and the app must not proceed with the call.
    pub fn report_incoming<F>(
        &self,
        call: IncomingCall,
        completion: F,
    ) -> Result<(), CallProviderError>
    where
        F: FnOnce(Result<(), CallProviderError>) + 'static,
    {
        let provider = self
            .provider
            .as_ref()
            .ok_or(CallProviderError::ProviderUnavailable)?;
        call.validate(&self.config)?;

        let uuid = native_uuid(call.id)?;
        let update = native_call_update(&call);
        let generation = self
            .shared
            .calls
            .borrow_mut()
            .begin_report(call.id)
            .map_err(|error| match error {
                BeginReportError::Duplicate => CallProviderError::DuplicateCallId,
                BeginReportError::Unavailable => CallProviderError::ProviderUnavailable,
            })?;

        let callback_context = Arc::new(MainThreadBound::new(
            ReportCallbackContext {
                shared: Rc::clone(&self.shared),
                weak_provider: Weak::from_retained(provider),
                call_id: call.id,
                generation,
                uuid: uuid.clone(),
                completion: Rc::new(RefCell::new(Some(Box::new(completion) as ReportCompletion))),
            },
            self._main_thread,
        ));
        let callback_context = Arc::new(OnceSlot::new(callback_context));
        let report_block = RcBlock::new(move |native_error: *mut NSError| {
            let accepted = native_error.is_null();
            let Some(context) = callback_context.take() else {
                return;
            };
            DispatchQueue::main().exec_async(move || {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    // SAFETY: This closure is submitted to the main dispatch queue.
                    let main_thread = unsafe { MainThreadMarker::new_unchecked() };
                    complete_incoming_report(context.get(main_thread), accepted);
                }));
            });
        });
        // SAFETY: The block only accesses its synchronized one-shot context on the callback queue,
        // then sends all provider state and user callback access asynchronously to the main queue.
        unsafe {
            provider.reportNewIncomingCallWithUUID_update_completion(&uuid, &update, &report_block)
        };
        Ok(())
    }

    /// Report a call as ended or defer that report until an incoming report completes.
    pub fn end_call(
        &self,
        call_id: CallId,
        reason: CallEndReason,
    ) -> Result<EndCallOutcome, CallProviderError> {
        let provider = self
            .provider
            .as_ref()
            .ok_or(CallProviderError::ProviderUnavailable)?;
        match self.shared.calls.borrow_mut().end(call_id, reason) {
            EndTransition::ReportNow(reason) => {
                let uuid = native_uuid(call_id)?;
                report_native_end(provider, &uuid, reason);
                Ok(EndCallOutcome::Reported)
            }
            EndTransition::Deferred => Ok(EndCallOutcome::DeferredUntilReport),
            EndTransition::AlreadyEnding => Ok(EndCallOutcome::AlreadyEnding),
            EndTransition::NotFound => Ok(EndCallOutcome::NotFound),
            EndTransition::Unavailable => Err(CallProviderError::ProviderUnavailable),
        }
    }

    /// Invalidate and release this provider. The operation is idempotent.
    pub fn invalidate(&mut self) {
        if self.provider.is_none() {
            return;
        }
        self.shared.calls.borrow_mut().invalidate();
        if let Some(provider) = self.provider.as_ref() {
            // SAFETY: Invalidation is required before provider release; this owner and its
            // MainThreadOnly delegate are both confined to the main thread.
            unsafe { provider.invalidate() };
            // SAFETY: Detaching after invalidation prevents any later weak-delegate callback.
            unsafe { provider.setDelegate_queue(None, None) };
        }
        self.delegate.take();
        self.delegate_queue.take();
        self.provider.take();
    }
}

impl Drop for CallProvider {
    fn drop(&mut self) {
        self.invalidate();
    }
}

fn native_provider_config(
    config: &CallProviderConfig,
) -> Result<Retained<CXProviderConfiguration>, CallProviderError> {
    if config.includes_calls_in_recents.is_some() && !available!(ios = 11.0) {
        return Err(CallProviderError::UnavailableOption);
    }
    let localized_name = NSString::from_str(&config.localized_name);
    let native = if available!(ios = 14.0) {
        // SAFETY: CXProviderConfiguration.init is available starting in iOS 14.
        unsafe { CXProviderConfiguration::new() }
    } else {
        // SAFETY: The public deprecated initializer is the supported iOS 10–13 path.
        #[allow(deprecated)]
        unsafe {
            CXProviderConfiguration::initWithLocalizedName(
                CXProviderConfiguration::alloc(),
                &localized_name,
            )
        }
    };

    let native_handles = config
        .supported_handles
        .iter()
        .map(|kind| NSNumber::new_isize(native_handle_type(*kind).0 as isize))
        .collect::<Vec<_>>();
    let handle_set = NSSet::from_retained_slice(&native_handles);
    unsafe {
        native.setSupportedHandleTypes(&handle_set);
        native.setSupportsVideo(config.supports_video);
        native.setMaximumCallGroups(config.maximum_call_groups as _);
        native.setMaximumCallsPerCallGroup(config.maximum_calls_per_group as _);
        if let Some(includes) = config.includes_calls_in_recents {
            // SAFETY: The availability guard above establishes the iOS 11 minimum.
            native.setIncludesCallsInRecents(includes);
        }
    }
    Ok(native)
}

fn native_call_update(call: &IncomingCall) -> Retained<CXCallUpdate> {
    let handle_value = NSString::from_str(&call.handle);
    let handle = unsafe {
        CXHandle::initWithType_value(
            CXHandle::alloc(),
            native_handle_type(call.handle_kind),
            &handle_value,
        )
    };
    let update = unsafe { CXCallUpdate::new() };
    unsafe {
        update.setRemoteHandle(Some(&handle));
        update.setHasVideo(call.has_video);
        update.setSupportsHolding(false);
        update.setSupportsGrouping(false);
        update.setSupportsUngrouping(false);
        update.setSupportsDTMF(false);
        if let Some(display_name) = call.display_name.as_deref() {
            let name = NSString::from_str(display_name);
            update.setLocalizedCallerName(Some(&name));
        }
    }
    update
}

fn native_handle_type(kind: CallHandleKind) -> CXHandleType {
    match kind {
        CallHandleKind::Generic => CXHandleType::Generic,
        CallHandleKind::PhoneNumber => CXHandleType::PhoneNumber,
        CallHandleKind::EmailAddress => CXHandleType::EmailAddress,
    }
}

fn native_uuid(id: CallId) -> Result<Retained<NSUUID>, CallProviderError> {
    let value = NSString::from_str(&id.uuid_string());
    NSUUID::from_string(&value).ok_or(CallProviderError::NativeConversionFailed)
}

fn call_id_from_native(uuid: &NSUUID) -> Option<CallId> {
    let value = uuid.UUIDString().to_string();
    CallId::from_uuid_string(&value)
}

fn report_native_end(provider: &CXProvider, uuid: &NSUUID, reason: CallEndReason) {
    unsafe {
        provider.reportCallWithUUID_endedAtDate_reason(uuid, None, native_end_reason(reason));
    }
}

fn native_end_reason(reason: CallEndReason) -> CXCallEndedReason {
    match reason {
        CallEndReason::RemoteEnded => CXCallEndedReason::RemoteEnded,
        CallEndReason::Failed => CXCallEndedReason::Failed,
        CallEndReason::Unanswered => CXCallEndedReason::Unanswered,
        CallEndReason::AnsweredElsewhere => CXCallEndedReason::AnsweredElsewhere,
        CallEndReason::DeclinedElsewhere => CXCallEndedReason::DeclinedElsewhere,
    }
}

fn invoke_report_completion(
    callback: &Rc<RefCell<Option<ReportCompletion>>>,
    result: Result<(), CallProviderError>,
) {
    let callback = callback.borrow_mut().take();
    if let Some(callback) = callback {
        let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
    }
}

fn complete_incoming_report(context: &ReportCallbackContext, accepted: bool) {
    let transition = catch_unwind(AssertUnwindSafe(|| {
        context.shared.calls.borrow_mut().report_completed(
            context.call_id,
            context.generation,
            accepted,
        )
    }))
    .unwrap_or(ReportTransition::Stale);

    let result = match transition {
        ReportTransition::Accepted | ReportTransition::EndAfterAccept(_) => Ok(()),
        ReportTransition::Rejected => Err(CallProviderError::SystemRejected),
        ReportTransition::Stale => Err(CallProviderError::ProviderUnavailable),
    };

    if let ReportTransition::EndAfterAccept(reason) = transition {
        if let Some(provider) = context.weak_provider.load() {
            report_native_end(&provider, &context.uuid, reason);
        }
    }
    invoke_report_completion(&context.completion, result);
}
