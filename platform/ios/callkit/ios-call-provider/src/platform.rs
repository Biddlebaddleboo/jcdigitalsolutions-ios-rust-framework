use std::cell::RefCell;
use std::collections::HashMap;
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
use objc2::{AnyThread, ClassType, DefinedClass, available, define_class, msg_send};
use objc2_call_kit::{
    CXAction, CXAnswerCallAction, CXCallAction, CXCallEndedReason, CXCallUpdate, CXEndCallAction,
    CXHandle, CXHandleType, CXProvider, CXProviderConfiguration, CXProviderDelegate,
    CXSetHeldCallAction, CXSetMutedCallAction,
};
use objc2_foundation::{NSError, NSNumber, NSObject, NSObjectProtocol, NSSet, NSString, NSUUID};

use crate::registry::{
    BeginReportError, CallRegistry, DeferredActionGate, EndTransition, OnceSlot, ReportTransition,
};
use crate::{
    ActionDecision, CallActionCompletion, CallActionCompletionError, CallActionKind,
    CallActionRequest, CallEndReason, CallHandleKind, CallId, CallProviderConfig,
    CallProviderError, CallProviderEvent, EndCallOutcome, IncomingCall,
};

type EventHandler = Box<dyn FnMut(CallProviderEvent) -> ActionDecision + 'static>;
type LifecycleHandler = Box<dyn FnMut(CallProviderEvent) + 'static>;
type DeferredActionHandler = Box<dyn FnMut(CallActionRequest) + 'static>;
type ReportCompletion = Box<dyn FnOnce(Result<(), CallProviderError>) + 'static>;

struct ProviderShared {
    state: Arc<MainThreadBound<RefCell<ProviderState>>>,
    handler: RefCell<Option<EventHandler>>,
    lifecycle_handler: RefCell<Option<LifecycleHandler>>,
    action_handler: RefCell<Option<DeferredActionHandler>>,
}

#[derive(Default)]
struct ProviderState {
    calls: CallRegistry,
    pending_actions: PendingActionRegistry,
}

struct PendingNativeAction {
    call_id: CallId,
    kind: CallActionKind,
    gate: Arc<DeferredActionGate>,
    native: Retained<CXCallAction>,
    completed: bool,
}

impl PendingNativeAction {
    fn complete(&mut self, fulfill: bool) {
        if self.completed {
            return;
        }
        if !unsafe { self.native.isComplete() } {
            self.completed = true;
            if fulfill {
                unsafe { self.native.fulfill() };
            } else {
                unsafe { self.native.fail() };
            }
        } else {
            self.completed = true;
        }
    }
}

impl Drop for PendingNativeAction {
    fn drop(&mut self) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: Pending actions are created, removed, and dropped only on the provider's
            // main queue because their registry lives in `MainThreadBound` state.
            if !self.completed && !unsafe { self.native.isComplete() } {
                self.completed = true;
                unsafe { self.native.fail() };
            }
        }));
    }
}

#[derive(Default)]
struct PendingActionRegistry {
    actions: HashMap<String, PendingNativeAction>,
}

impl PendingActionRegistry {
    fn insert(
        &mut self,
        id: String,
        action: PendingNativeAction,
    ) -> Result<(), PendingNativeAction> {
        if self.actions.contains_key(&id) {
            return Err(action);
        }
        self.actions.insert(id, action);
        Ok(())
    }

    fn take(&mut self, id: &str, gate: &Arc<DeferredActionGate>) -> Option<PendingNativeAction> {
        if !self
            .actions
            .get(id)
            .is_some_and(|pending| Arc::ptr_eq(&pending.gate, gate))
        {
            return None;
        }
        self.actions.remove(id)
    }

    fn timeout(&mut self, id: &str) -> Option<PendingNativeAction> {
        self.actions.remove(id).inspect(|pending| {
            pending.gate.expire();
        })
    }

    fn expire_call(&mut self, call_id: CallId) -> Vec<PendingNativeAction> {
        let ids = self
            .actions
            .iter()
            .filter(|&(_, pending)| pending.call_id == call_id)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        ids.into_iter()
            .filter_map(|id| self.actions.remove(&id))
            .inspect(|pending| pending.gate.expire())
            .collect()
    }

    fn invalidate(&mut self) -> Vec<PendingNativeAction> {
        std::mem::take(&mut self.actions)
            .into_values()
            .inspect(|pending| pending.gate.invalidate_unavailable())
            .collect()
    }
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
                let shared = &self.ivars().shared;
                // SAFETY: CallKit invokes this delegate on the explicitly selected main queue.
                let main_thread = unsafe { MainThreadMarker::new_unchecked() };
                let pending = {
                    let mut state = shared.state.get(main_thread).borrow_mut();
                    state.calls.reset();
                    state.pending_actions.invalidate()
                };
                drop(pending);
                dispatch_lifecycle(shared, CallProviderEvent::ProviderReset);
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
                    handle_call_action(&self.ivars().shared, action, CallActionKind::Answer);
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
                    handle_call_action(&self.ivars().shared, action, CallActionKind::End);
                });
            }));
        }

        #[unsafe(method(provider:performSetMutedCallAction:))]
        unsafe fn provider_performSetMutedCallAction(
            &self,
            _provider: &CXProvider,
            action: &CXSetMutedCallAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    let is_muted = unsafe { action.isMuted() };
                    handle_call_action(
                        &self.ivars().shared,
                        action.as_super(),
                        CallActionKind::SetMuted { is_muted },
                    );
                });
            }));
        }

        #[unsafe(method(provider:performSetHeldCallAction:))]
        unsafe fn provider_performSetHeldCallAction(
            &self,
            _provider: &CXProvider,
            action: &CXSetHeldCallAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    let is_held = unsafe { action.isOnHold() };
                    handle_call_action(
                        &self.ivars().shared,
                        action.as_super(),
                        CallActionKind::SetHeld { is_held },
                    );
                });
            }));
        }

        #[unsafe(method(provider:timedOutPerformingAction:))]
        unsafe fn provider_timedOutPerformingAction(
            &self,
            _provider: &CXProvider,
            action: &CXAction,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let shared = &self.ivars().shared;
                // SAFETY: CallKit invokes this delegate on the explicitly selected main queue.
                let main_thread = unsafe { MainThreadMarker::new_unchecked() };
                let action_id = native_action_id(action);
                let pending = shared
                    .state
                    .get(main_thread)
                    .borrow_mut()
                    .pending_actions
                    .timeout(&action_id);
                drop(pending);
                dispatch_lifecycle(shared, CallProviderEvent::ActionTimedOut);
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

fn dispatch_event(shared: &ProviderShared, event: CallProviderEvent) -> ActionDecision {
    let mut handler = shared.handler.borrow_mut();
    let Some(handler) = handler.as_mut() else {
        return ActionDecision::Fail;
    };
    catch_unwind(AssertUnwindSafe(|| handler(event))).unwrap_or(ActionDecision::Fail)
}

fn dispatch_lifecycle(shared: &ProviderShared, event: CallProviderEvent) {
    if let Some(handler) = shared.handler.borrow_mut().as_mut() {
        let _ = catch_unwind(AssertUnwindSafe(|| handler(event)));
    } else if let Some(handler) = shared.lifecycle_handler.borrow_mut().as_mut() {
        let _ = catch_unwind(AssertUnwindSafe(|| handler(event)));
    }
}

fn handle_call_action(shared: &ProviderShared, action: &CXCallAction, kind: CallActionKind) {
    if shared.handler.borrow().is_some() {
        complete_call_action(shared, action, kind);
    } else {
        defer_call_action(shared, action, kind);
    }
}

fn complete_call_action(shared: &ProviderShared, action: &CXCallAction, kind: CallActionKind) {
    let id = unsafe { action.callUUID() };
    let Some(call_id) = call_id_from_native(&id) else {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    };
    // SAFETY: This callback is invoked by CallKit on the selected main queue.
    let main_thread = unsafe { MainThreadMarker::new_unchecked() };
    if !shared
        .state
        .get(main_thread)
        .borrow()
        .calls
        .is_active(call_id)
    {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    }

    let event = kind.provider_event(call_id);
    let decision = dispatch_event(shared, event);

    // CallKit may have timed out the action while user code ran. Timed-out actions are already
    // failed by the system and must not be completed again.
    if unsafe { action.isComplete() } {
        return;
    }
    match decision {
        ActionDecision::Fulfill => {
            if matches!(kind, CallActionKind::End) {
                shared
                    .state
                    .get(main_thread)
                    .borrow_mut()
                    .calls
                    .finish_end_action(call_id);
            }
            unsafe { action.fulfill() };
        }
        ActionDecision::Fail => unsafe { action.fail() },
    }
}

fn defer_call_action(shared: &ProviderShared, action: &CXCallAction, kind: CallActionKind) {
    let call_uuid = unsafe { action.callUUID() };
    let Some(call_id) = call_id_from_native(&call_uuid) else {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    };
    // SAFETY: This function is called only from a CallKit delegate on the main queue.
    let main_thread = unsafe { MainThreadMarker::new_unchecked() };
    if !shared
        .state
        .get(main_thread)
        .borrow()
        .calls
        .is_active(call_id)
    {
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    }

    let action_id = native_action_id(action);
    let gate = Arc::new(DeferredActionGate::default());
    let pending = PendingNativeAction {
        call_id,
        kind,
        gate: Arc::clone(&gate),
        native: objc2::Message::retain(action),
        completed: false,
    };
    let inserted = shared
        .state
        .get(main_thread)
        .borrow_mut()
        .pending_actions
        .insert(action_id.clone(), pending);
    if let Err(pending) = inserted {
        drop(pending);
        if !unsafe { action.isComplete() } {
            unsafe { action.fail() };
        }
        return;
    }

    let state = Arc::downgrade(&shared.state);
    let completion_gate = Arc::clone(&gate);
    let completion = CallActionCompletion::new(move |decision| {
        completion_gate.submit()?;
        let Some(state) = state.upgrade() else {
            completion_gate.invalidate_unavailable();
            return Err(CallActionCompletionError::ProviderUnavailable);
        };
        let gate_on_main = Arc::clone(&completion_gate);
        DispatchQueue::main().exec_async(move || {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: This closure is submitted to the main dispatch queue.
                let main_thread = unsafe { MainThreadMarker::new_unchecked() };
                resolve_deferred_action(
                    state.get(main_thread),
                    &action_id,
                    &gate_on_main,
                    decision,
                );
            }));
        });
        Ok(())
    });
    let request = CallActionRequest::new(call_id, kind, completion);
    if let Some(handler) = shared.action_handler.borrow_mut().as_mut() {
        let _ = catch_unwind(AssertUnwindSafe(|| handler(request)));
    } else {
        drop(request);
    }
}

fn resolve_deferred_action(
    state: &RefCell<ProviderState>,
    action_id: &str,
    gate: &Arc<DeferredActionGate>,
    decision: ActionDecision,
) {
    let mut initial_state = state.borrow_mut();
    let Some(mut pending) = initial_state.pending_actions.take(action_id, gate) else {
        gate.expire();
        return;
    };
    if !gate.begin_resolution() {
        drop(initial_state);
        return;
    }
    drop(initial_state);
    if unsafe { pending.native.isComplete() } {
        gate.expire();
        return;
    }

    let mut state = state.borrow_mut();
    let (should_fulfill, expired_actions) = match decision {
        ActionDecision::Fail => (false, Vec::new()),
        ActionDecision::Fulfill => match pending.kind {
            CallActionKind::Answer => {
                // The call may have ended while application work was deferred.
                (state.calls.is_active(pending.call_id), Vec::new())
            }
            CallActionKind::End => {
                let ended = state.calls.finish_end_action(pending.call_id);
                let expired = if ended {
                    state.pending_actions.expire_call(pending.call_id)
                } else {
                    Vec::new()
                };
                (ended, expired)
            }
            CallActionKind::SetMuted { .. } | CallActionKind::SetHeld { .. } => {
                (state.calls.is_active(pending.call_id), Vec::new())
            }
        },
    };
    drop(state);
    drop(expired_actions);
    pending.complete(should_fulfill);
    let _ = gate.resolve();
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
    /// Create a provider on the main thread with an immediate call-action handler.
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
        Self::with_handlers(main_thread, config, Some(Box::new(handler)), None, None)
    }

    /// Create a provider whose call actions can resolve after application work completes.
    ///
    /// The lifecycle handler receives only `ProviderReset` and `ActionTimedOut`. The action
    /// handler receives a one-shot request on the main CallKit delegate queue and must return
    /// promptly. Move the request or its completion to app-owned asynchronous work; this API adds
    /// no executor or networking policy. Dropping an unsubmitted completion fails the action.
    pub fn new_with_deferred_actions<L, A>(
        main_thread: MainThreadMarker,
        config: CallProviderConfig,
        lifecycle_handler: L,
        action_handler: A,
    ) -> Result<Self, CallProviderError>
    where
        L: FnMut(CallProviderEvent) + 'static,
        A: FnMut(CallActionRequest) + 'static,
    {
        Self::with_handlers(
            main_thread,
            config,
            None,
            Some(Box::new(lifecycle_handler)),
            Some(Box::new(action_handler)),
        )
    }

    fn with_handlers(
        main_thread: MainThreadMarker,
        config: CallProviderConfig,
        handler: Option<EventHandler>,
        lifecycle_handler: Option<LifecycleHandler>,
        action_handler: Option<DeferredActionHandler>,
    ) -> Result<Self, CallProviderError> {
        config.validate()?;
        let native_config = native_provider_config(&config)?;
        let provider =
            unsafe { CXProvider::initWithConfiguration(CXProvider::alloc(), &native_config) };
        let state = Arc::new(MainThreadBound::new(
            RefCell::new(ProviderState::default()),
            main_thread,
        ));
        let shared = Rc::new(ProviderShared {
            state,
            handler: RefCell::new(handler),
            lifecycle_handler: RefCell::new(lifecycle_handler),
            action_handler: RefCell::new(action_handler),
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
            .state
            .get(self._main_thread)
            .borrow_mut()
            .calls
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
        let transition = self
            .shared
            .state
            .get(self._main_thread)
            .borrow_mut()
            .calls
            .end(call_id, reason);
        match transition {
            EndTransition::ReportNow(reason) => {
                let pending = self
                    .shared
                    .state
                    .get(self._main_thread)
                    .borrow_mut()
                    .pending_actions
                    .expire_call(call_id);
                drop(pending);
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
        let mut state = self.shared.state.get(self._main_thread).borrow_mut();
        state.calls.invalidate();
        let pending = state.pending_actions.invalidate();
        drop(state);
        drop(pending);
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

fn native_action_id(action: &CXAction) -> String {
    unsafe { action.UUID() }.UUIDString().to_string()
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
    // SAFETY: This callback is invoked only after dispatch to the main queue.
    let main_thread = unsafe { MainThreadMarker::new_unchecked() };
    let transition = catch_unwind(AssertUnwindSafe(|| {
        context
            .shared
            .state
            .get(main_thread)
            .borrow_mut()
            .calls
            .report_completed(context.call_id, context.generation, accepted)
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
