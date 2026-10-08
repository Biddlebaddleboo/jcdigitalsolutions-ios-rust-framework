use std::char;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::string::String;
use std::sync::Arc;

use framework_notifications::NotificationId;
use framework_notifications::response::{
    NotificationActionId, NotificationResponse, NotificationResponseKind,
};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
use objc2_user_notifications::{
    UNNotificationDefaultActionIdentifier, UNNotificationDismissActionIdentifier,
    UNNotificationRequest, UNNotificationResponse, UNPushNotificationTrigger,
    UNTextInputNotificationResponse, UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

type ResponseClosure = Arc<dyn Fn(NotificationResponse) + Send + Sync + 'static>;

/// A reason why response-delegate install did not occur
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum InstallError {
    /// The shared notification center already has a delegate
    DelegateAlreadySet,
}

/// A handle for the opt-in local-notification response delegate
///
/// Keep this value alive while the app should receive local notification responses. The native
/// center stores its delegate weakly, so drop of this value also releases the Rust delegate
///
/// Install before app launch completes and serialize this call with all reads or writes of
/// the shared center delegate. The native property is weak and non-atomic, so external concurrent
/// mutation cannot be made race-free by this crate
pub struct IosNotificationResponses {
    _delegate: Retained<ResponseDelegate>,
}

impl IosNotificationResponses {
    /// Install a synchronous response closure if the shared center has no delegate
    ///
    /// The closure runs on the native delegate callback thread, whose queue Apple does not
    /// specify. It must be safe to call from any thread and should return promptly. Rust panics
    /// are caught and discarded so the native completion still runs
    ///
    /// This API does not replace a delegate present at the time of the initial check. Apple marks
    /// the delegate property weak and non-atomic; the app must serialize all delegate access while
    /// this call runs and must call it before launch completes
    pub fn install<F>(handler: F) -> Result<Self, InstallError>
    where
        F: Fn(NotificationResponse) + Send + Sync + 'static,
    {
        autoreleasepool(|_| {
            let center = UNUserNotificationCenter::currentNotificationCenter();
            if center.delegate().is_some() {
                return Err(InstallError::DelegateAlreadySet);
            }

            let delegate = ResponseDelegate::new(Arc::new(handler));
            let delegate_object = ProtocolObject::from_ref(&*delegate);
            center.setDelegate(Some(delegate_object));
            Ok(Self {
                _delegate: delegate,
            })
        })
    }
}

struct ResponseDelegateIvars {
    handler: ResponseClosure,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = ResponseDelegateIvars]
    struct ResponseDelegate;

    // SAFETY: This class derives from NSObject and stores only a thread-safe Rust closure
    unsafe impl NSObjectProtocol for ResponseDelegate {}

    // SAFETY: The callback can run on any native queue; its Rust closure is thread-safe, its
    // completion runs once, and Rust panics are caught before the Objective-C boundary
    #[allow(non_snake_case)]
    unsafe impl UNUserNotificationCenterDelegate for ResponseDelegate {
        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        unsafe fn userNotificationCenter_didReceiveNotificationResponse_withCompletionHandler(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &block2::DynBlock<dyn Fn()>,
        ) {
            let mut completion = NativeCompletion::new(completion_handler);
            let _ = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: objc2 calls this method with a live `self` ref; this retain keeps its
                // ivars valid while the Rust closure can drop the public handle
                let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                    return;
                };
                let handler = Arc::clone(&self.ivars().handler);
                autoreleasepool(|_| {
                    if let Some(response) = response_value(response) {
                        handler(response);
                    }
                });
            }));
            completion.run();
        }
    }
);

impl ResponseDelegate {
    fn new(handler: ResponseClosure) -> Retained<Self> {
        let state = ResponseDelegateIvars { handler };
        let allocated = Self::alloc().set_ivars(state);
        // SAFETY: The class derives from NSObject and its ivars are set before NSObject init
        unsafe { msg_send![super(allocated), init] }
    }
}

struct NativeCompletion<'a> {
    block: &'a block2::DynBlock<dyn Fn()>,
    called: bool,
}

impl<'a> NativeCompletion<'a> {
    fn new(block: &'a block2::DynBlock<dyn Fn()>) -> Self {
        Self {
            block,
            called: false,
        }
    }

    fn run(&mut self) {
        if !self.called {
            self.called = true;
            let _ = catch_unwind(AssertUnwindSafe(|| self.block.call(())));
        }
    }
}

impl Drop for NativeCompletion<'_> {
    fn drop(&mut self) {
        self.run();
    }
}

unsafe fn retain_callback_receiver(
    receiver: &ResponseDelegate,
) -> Option<Retained<ResponseDelegate>> {
    // SAFETY: objc2 supplies a live `self` ref; one retain keeps the object alive through a
    // reentrant closure call
    unsafe { Retained::retain(receiver as *const ResponseDelegate as *mut ResponseDelegate) }
}

fn response_value(response: &UNNotificationResponse) -> Option<NotificationResponse> {
    let notification = response.notification();
    let request: Retained<UNNotificationRequest> = notification.request();
    if request.trigger().is_some_and(|trigger| {
        trigger
            .downcast_ref::<UNPushNotificationTrigger>()
            .is_some()
    }) {
        return None;
    }

    let notification_id = NotificationId::new(owned_string(&request.identifier())?).ok()?;
    let action_identifier = response.actionIdentifier();
    let kind = if is_default_action(&action_identifier) {
        NotificationResponseKind::Default
    } else if is_dismiss_action(&action_identifier) {
        NotificationResponseKind::Dismiss
    } else if let Some(text_response) = response.downcast_ref::<UNTextInputNotificationResponse>() {
        NotificationResponseKind::TextInput {
            action_id: NotificationActionId::new(owned_string(&action_identifier)?).ok()?,
            text: owned_string(&text_response.userText())?,
        }
    } else {
        NotificationResponseKind::CustomAction(
            NotificationActionId::new(owned_string(&action_identifier)?).ok()?,
        )
    };

    Some(NotificationResponse::new(notification_id, kind))
}

fn is_default_action(action_identifier: &NSString) -> bool {
    // SAFETY: Apple exports this immutable system action identifier
    unsafe { action_identifier.isEqualToString(UNNotificationDefaultActionIdentifier) }
}

fn is_dismiss_action(action_identifier: &NSString) -> bool {
    // SAFETY: Apple exports this immutable system action identifier
    unsafe { action_identifier.isEqualToString(UNNotificationDismissActionIdentifier) }
}

fn owned_string(value: &NSString) -> Option<String> {
    let utf16 = (0..value.len_utf16()).map(|index| value.characterAtIndex(index));
    char::decode_utf16(utf16)
        .collect::<Result<String, _>>()
        .ok()
}
