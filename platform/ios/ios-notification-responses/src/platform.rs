use std::panic::{AssertUnwindSafe, catch_unwind};
use std::string::String;
use std::sync::Arc;
use std::vec::Vec;

use crate::presentation::ForegroundPresentationOptions;
use crate::response_user_info::{decode_utf16, sorted_string_entries};
use framework_notifications::NotificationId;
use framework_notifications::response::{
    NotificationActionId, NotificationResponse, NotificationResponseKind,
};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2_foundation::{
    NSObject, NSObjectProtocol, NSOperatingSystemVersion, NSProcessInfo, NSString,
};
use objc2_user_notifications::{
    UNNotification, UNNotificationContent, UNNotificationDefaultActionIdentifier,
    UNNotificationDismissActionIdentifier, UNNotificationPresentationOptions,
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
        Self::install_inner(handler, ForegroundPresentationOptions::new())
    }

    /// Install a synchronous response closure and select foreground notification presentation
    ///
    /// The presentation options apply to notifications delivered while the app is in the
    /// foreground. A requested banner uses the legacy alert style on iOS 10 through iOS 13,
    /// because the banner option is available only on iOS 14 and later. Response closure
    /// threading, ownership, and completion behavior match [`Self::install`]
    pub fn install_with_foreground_presentation<F>(
        handler: F,
        presentation: ForegroundPresentationOptions,
    ) -> Result<Self, InstallError>
    where
        F: Fn(NotificationResponse) + Send + Sync + 'static,
    {
        Self::install_inner(handler, presentation)
    }

    fn install_inner<F>(
        handler: F,
        presentation: ForegroundPresentationOptions,
    ) -> Result<Self, InstallError>
    where
        F: Fn(NotificationResponse) + Send + Sync + 'static,
    {
        autoreleasepool(|_| {
            let center = UNUserNotificationCenter::currentNotificationCenter();
            if center.delegate().is_some() {
                return Err(InstallError::DelegateAlreadySet);
            }

            let delegate = ResponseDelegate::new(Arc::new(handler), presentation);
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
    presentation: ForegroundPresentationOptions,
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
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        unsafe fn userNotificationCenter_willPresentNotification_withCompletionHandler(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            let mut completion = NativePresentationCompletion::new(completion_handler);
            let presentation = catch_unwind(AssertUnwindSafe(|| {
                self.ivars().presentation.native_options()
            }))
            .unwrap_or_else(|_| UNNotificationPresentationOptions::empty());
            completion.run(presentation);
        }

        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        unsafe fn userNotificationCenter_didReceiveNotificationResponse_withCompletionHandler(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &block2::DynBlock<dyn Fn()>,
        ) {
            let mut completion = NativeCompletion::new(completion_handler);
            // SAFETY: objc2 calls this method with a live `self` ref; this retain keeps its
            // ivars valid through the Rust closure and native completion call
            let receiver = catch_unwind(AssertUnwindSafe(|| unsafe {
                retain_callback_receiver(self)
            }))
            .ok()
            .flatten();
            let _ = catch_unwind(AssertUnwindSafe(|| {
                if receiver.is_none() {
                    return;
                }
                let handler = Arc::clone(&self.ivars().handler);
                autoreleasepool(|_| {
                    if let Some(response) = response_value(response) {
                        handler(response);
                    }
                });
            }));
            completion.run();
            let _ = catch_unwind(AssertUnwindSafe(move || drop(receiver)));
        }
    }
);

impl ResponseDelegate {
    fn new(
        handler: ResponseClosure,
        presentation: ForegroundPresentationOptions,
    ) -> Retained<Self> {
        let state = ResponseDelegateIvars {
            handler,
            presentation,
        };
        let allocated = Self::alloc().set_ivars(state);
        // SAFETY: The class derives from NSObject and its ivars are set before NSObject init
        unsafe { msg_send![super(allocated), init] }
    }
}

impl ForegroundPresentationOptions {
    fn native_options(self) -> UNNotificationPresentationOptions {
        let supports_banner = self.banner_requested()
            && NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(
                NSOperatingSystemVersion {
                    majorVersion: 14,
                    minorVersion: 0,
                    patchVersion: 0,
                },
            );
        let selection = self.for_system(supports_banner);
        let mut options = UNNotificationPresentationOptions::empty();
        if selection.banner {
            options |= UNNotificationPresentationOptions::Banner;
        }
        if selection.legacy_alert {
            #[allow(deprecated)]
            {
                options |= UNNotificationPresentationOptions::Alert;
            }
        }
        if selection.sound {
            options |= UNNotificationPresentationOptions::Sound;
        }
        if selection.badge {
            options |= UNNotificationPresentationOptions::Badge;
        }
        options
    }
}

struct NativePresentationCompletion<'a> {
    block: &'a block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
    called: bool,
}

impl<'a> NativePresentationCompletion<'a> {
    fn new(block: &'a block2::DynBlock<dyn Fn(UNNotificationPresentationOptions)>) -> Self {
        Self {
            block,
            called: false,
        }
    }

    fn run(&mut self, options: UNNotificationPresentationOptions) {
        if !self.called {
            self.called = true;
            let _ = catch_unwind(AssertUnwindSafe(|| self.block.call((options,))));
        }
    }
}

impl Drop for NativePresentationCompletion<'_> {
    fn drop(&mut self) {
        self.run(UNNotificationPresentationOptions::empty());
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
    let content = request.content();
    let user_info_entries = response_user_info(&content);
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

    Some(user_info_entries.into_iter().fold(
        NotificationResponse::new(notification_id, kind),
        |response, (key, value)| response.with_user_info_entry(key, value),
    ))
}

fn response_user_info(content: &UNNotificationContent) -> Vec<(String, String)> {
    let user_info = content.userInfo();
    let keys = user_info.allKeys();
    let entries = (0..keys.count()).map(|index| {
        let native_key = keys.objectAtIndex(index);
        let Some(native_key_string) = native_key.downcast_ref::<NSString>() else {
            return (None, None);
        };
        let Some(key) = owned_string(native_key_string) else {
            return (None, None);
        };
        let Some(native_value) = user_info.objectForKey(&native_key) else {
            return (Some(key), None);
        };
        let value = native_value
            .downcast_ref::<NSString>()
            .and_then(owned_string);
        (Some(key), value)
    });
    sorted_string_entries(entries)
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
    decode_utf16(utf16)
}
