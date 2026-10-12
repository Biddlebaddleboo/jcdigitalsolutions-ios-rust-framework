use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;

use block2::DynBlock;
use objc2::MainThreadMarker;
use objc2::MainThreadOnly;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_foundation::{
    NSData, NSDictionary, NSJSONSerialization, NSJSONWritingOptions, NSObject, NSObjectProtocol,
    NSSet,
};
use objc2_push_kit::{
    PKPushCredentials, PKPushPayload, PKPushRegistry, PKPushRegistryDelegate, PKPushType,
    PKPushTypeVoIP,
};

use crate::VoipPushToken;
use crate::completion::OneShotCompletion;
use ios_call_provider::{CallProvider, CallProviderError, IncomingCall};

type EventHandler = Box<dyn FnMut(PushKitEvent) + 'static>;

/// A failure to serialize the native APNs payload as JSON bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadEncodingError {
    /// Foundation could not serialize the PushKit dictionary as JSON.
    JsonSerializationFailed,
}

/// A PushKit event for the caller-selected VoIP type.
#[derive(Debug)]
#[non_exhaustive]
pub enum PushKitEvent {
    /// PushKit supplied a new opaque VoIP token.
    TokenUpdated(VoipPushToken),
    /// PushKit invalidated the previously supplied VoIP token.
    TokenInvalidated,
    /// PushKit delivered a VoIP payload and its native completion handler.
    Incoming(IncomingVoipPush),
}

/// An owned JSON payload and one-shot PushKit completion for an incoming VoIP push.
///
/// The payload bytes are a UTF-8 JSON serialization of Apple's payload dictionary. The app owns
/// the schema. On iOS 13 SDK-linked apps, report the corresponding call with
/// `ios_call_provider::CallProvider::report_incoming` before calling
/// [`finish_after_callkit_report`](Self::finish_after_callkit_report).
#[derive(Debug)]
pub struct IncomingVoipPush {
    payload_json: Result<Vec<u8>, PayloadEncodingError>,
    completion: PushCompletion,
}

impl IncomingVoipPush {
    /// Returns the owned APNs JSON bytes, or the serialization failure.
    pub fn payload_json(&self) -> Result<&[u8], PayloadEncodingError> {
        self.payload_json.as_deref().map_err(|error| *error)
    }

    /// Reports a parsed incoming call through the shared CallKit provider, then completes PushKit.
    ///
    /// The native PushKit completion runs only after the CallKit report callback. The supplied
    /// callback receives the same report result afterward. A synchronous report error returns
    /// directly and dropping the event invokes PushKit completion as a fallback.
    pub fn report_incoming<F>(
        self,
        provider: &CallProvider,
        call: IncomingCall,
        completion: F,
    ) -> Result<(), CallProviderError>
    where
        F: FnOnce(Result<(), CallProviderError>) + 'static,
    {
        let push_completion = self.completion;
        provider.report_incoming(call, move |result| {
            push_completion.finish();
            completion(result);
        })
    }

    /// Completes PushKit after the caller's CallKit incoming-report completion.
    ///
    /// Dropping this value also completes PushKit once as a fallback, including when the event
    /// handler unwinds. The fallback prevents a native completion leak; it does not satisfy the
    /// app's separate requirement to report a VoIP call to CallKit.
    pub fn finish_after_callkit_report(self) {
        self.completion.finish();
    }
}

/// A reason why the PushKit VoIP service could not be created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PushKitError {
    /// The iOS release predates the completion-bearing incoming-push delegate callback.
    UnsupportedOperatingSystem,
}

/// A retained iOS VoIP PushKit registry and its weak native delegate.
///
/// Create and retain this value at app launch. Its event callback runs on the main queue and must
/// return promptly. The service requests only the VoIP push type. A token event proves only that
/// PushKit supplied bytes; it does not prove APNs provider registration or delivery. The host app
/// configures the push entitlement and `UIBackgroundModes` value `voip`.
pub struct IosVoipPushService {
    registry: Option<Retained<PKPushRegistry>>,
    delegate: Option<Retained<PushDelegate>>,
    _main_thread: MainThreadMarker,
}

impl IosVoipPushService {
    /// Registers for VoIP pushes on the main queue and retains the registry and delegate.
    ///
    /// Set the host's APNs entitlement and VoIP background mode before shipping. PushKit does not
    /// prompt for permission. Registration can asynchronously deliver a token event; this method
    /// does not report successful registration or APNs reachability.
    pub fn register<F>(main_thread: MainThreadMarker, handler: F) -> Result<Self, PushKitError>
    where
        F: FnMut(PushKitEvent) + 'static,
    {
        if !objc2::available!(ios = 11.0, ..) {
            return Err(PushKitError::UnsupportedOperatingSystem);
        }

        autoreleasepool(|_| {
            let delegate = PushDelegate::new(main_thread, Box::new(handler));
            // SAFETY: nil selects the documented main queue. The Rust delegate is main-thread-only
            // and is retained below before PushKit can deliver a callback.
            let registry = unsafe { PKPushRegistry::initWithQueue(PKPushRegistry::alloc(), None) };
            let delegate_object = ProtocolObject::from_ref(&*delegate);
            // SAFETY: The delegate is retained by `Self` before setting the weak native property.
            unsafe { registry.setDelegate(Some(delegate_object)) };
            // SAFETY: `PKPushTypeVoIP` is PushKit's documented immutable type constant.
            let desired = unsafe { NSSet::setWithObject(PKPushTypeVoIP) };
            // SAFETY: The registry's live delegate is already installed, as required by PushKit.
            unsafe { registry.setDesiredPushTypes(Some(&desired)) };

            Ok(Self {
                registry: Some(registry),
                delegate: Some(delegate),
                _main_thread: main_thread,
            })
        })
    }

    /// Stops VoIP registration and clears PushKit's weak delegate before release.
    ///
    /// Call this on the same main thread as [`Self::register`]. Repeated calls are harmless.
    pub fn invalidate(&mut self) {
        let Some(registry) = self.registry.take() else {
            return;
        };

        // SAFETY: The registry and its delegate are main-queue-bound by construction. Clear the
        // requested types while the retained delegate is still alive, then clear the weak link.
        unsafe {
            registry.setDesiredPushTypes(None);
            registry.setDelegate(None);
        }
        self.delegate.take();
        drop(registry);
    }
}

impl Drop for IosVoipPushService {
    fn drop(&mut self) {
        self.invalidate();
    }
}

/// A one-shot native incoming-push completion handle.
#[derive(Debug)]
struct PushCompletion {
    completion: OneShotCompletion,
}

impl PushCompletion {
    fn new(completion: &DynBlock<dyn Fn()>) -> Self {
        let completion = completion.copy();
        Self {
            completion: OneShotCompletion::new(move || completion.call(())),
        }
    }

    fn finish(self) {
        let mut completion = self.completion;
        completion.complete();
    }
}

struct PushDelegateIvars {
    handler: RefCell<EventHandler>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = PushDelegateIvars]
    struct PushDelegate;

    // SAFETY: NSObjectProtocol adds no methods, and this delegate is main-queue-confined.
    unsafe impl NSObjectProtocol for PushDelegate {}

    // SAFETY: PushKit is configured with the main queue. Each native callback catches Rust
    // panics; incoming completion ownership is one-shot and falls back to completion on drop.
    #[allow(non_snake_case)]
    unsafe impl PKPushRegistryDelegate for PushDelegate {
        #[unsafe(method(pushRegistry:didUpdatePushCredentials:forType:))]
        unsafe fn pushRegistry_didUpdatePushCredentials_forType(
            &self,
            _registry: &PKPushRegistry,
            credentials: &PKPushCredentials,
            _push_type: &PKPushType,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: PushKit provides a live credentials object for this callback.
                    let token = unsafe { credentials.token() };
                    self.emit(PushKitEvent::TokenUpdated(VoipPushToken::from_bytes(
                        copy_data_bytes(&token),
                    )));
                });
            }));
        }

        #[unsafe(method(pushRegistry:didInvalidatePushTokenForType:))]
        unsafe fn pushRegistry_didInvalidatePushTokenForType(
            &self,
            _registry: &PKPushRegistry,
            _push_type: &PKPushType,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.emit(PushKitEvent::TokenInvalidated);
            }));
        }

        #[unsafe(method(pushRegistry:didReceiveIncomingPushWithPayload:forType:withCompletionHandler:))]
        unsafe fn pushRegistry_didReceiveIncomingPushWithPayload_forType_withCompletionHandler(
            &self,
            _registry: &PKPushRegistry,
            payload: &PKPushPayload,
            _push_type: &PKPushType,
            completion: &DynBlock<dyn Fn()>,
        ) {
            let completion = PushCompletion::new(completion);
            let _ = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    let payload_json = encode_payload(payload);
                    self.emit(PushKitEvent::Incoming(IncomingVoipPush {
                        payload_json,
                        completion,
                    }));
                });
            }));
        }
    }
);

impl PushDelegate {
    fn new(main_thread: MainThreadMarker, handler: EventHandler) -> Retained<Self> {
        let allocated = main_thread.alloc::<Self>();
        let allocated = allocated.set_ivars(PushDelegateIvars {
            handler: RefCell::new(handler),
        });
        // SAFETY: The class derives from NSObject and its ivars are initialized before NSObject's
        // designated initializer runs.
        unsafe { msg_send![super(allocated), init] }
    }

    fn emit(&self, event: PushKitEvent) {
        let mut event = Some(event);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            if let Ok(mut handler) = self.ivars().handler.try_borrow_mut() {
                handler(event.take().expect("event is present"));
            }
        }));
        drop(event);
    }
}

fn encode_payload(payload: &PKPushPayload) -> Result<Vec<u8>, PayloadEncodingError> {
    // SAFETY: PushKit's `dictionaryPayload` is a Foundation JSON dictionary by contract.
    let dictionary: Retained<NSDictionary> = unsafe { payload.dictionaryPayload() };
    let value = &*dictionary;
    // SAFETY: The object is the live JSON dictionary returned by PushKit, and empty options request
    // a compact JSON serialization without altering payload values.
    let data = unsafe {
        NSJSONSerialization::dataWithJSONObject_options_error(value, NSJSONWritingOptions::empty())
    }
    .map_err(|_| PayloadEncodingError::JsonSerializationFailed)?;
    Ok(copy_data_bytes(&data))
}

fn copy_data_bytes(data: &NSData) -> Vec<u8> {
    let len = data.length();
    let mut bytes = vec![0; len];
    if len > 0 {
        let buffer = NonNull::new(bytes.as_mut_ptr().cast()).expect("non-empty Vec has a pointer");
        // SAFETY: The Vec has `len` writable bytes and remains alive for the native copy.
        unsafe { data.getBytes_length(buffer, len) };
    }
    bytes
}
