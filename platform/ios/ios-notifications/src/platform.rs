use crate::conversion::{
    UtcDateComponents, authorization_state_from_native, utc_trigger_components,
};
use crate::operation::CompletionCell;
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_core::{AuthorizationState, Availability, Error, ErrorKind, PlatformErrorCode};
use framework_notifications::{
    Notification, NotificationBackend, NotificationError, NotificationId,
};
use objc2::ClassType;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::Bool;
use objc2_foundation::{NSArray, NSCalendar, NSDateComponents, NSError, NSString, NSTimeZone};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNCalendarNotificationTrigger, UNMutableNotificationContent,
    UNNotificationRequest, UNNotificationSettings, UNUserNotificationCenter,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

type Completion<T> = Arc<CompletionCell<T, NotificationError>>;
type StartOperation<T, O> = fn(Retained<UNUserNotificationCenter>, O, Completion<T>);

/// iOS authorization-query future.
pub type IosAuthorizationFuture = IosNotificationsFuture<AuthorizationState, ()>;

/// iOS authorization-request future.
pub type IosRequestAuthorizationFuture = IosNotificationsFuture<AuthorizationState, ()>;

/// iOS notification-schedule future.
pub type IosScheduleFuture = IosNotificationsFuture<(), Notification>;

/// iOS pending-notification cancellation future.
pub type IosCancelFuture = IosNotificationsFuture<bool, NotificationId>;

/// A lazy UserNotifications operation that owns safe callback state until one terminal result.
pub struct IosNotificationsFuture<T, O> {
    center: Retained<UNUserNotificationCenter>,
    operation: Option<O>,
    start: StartOperation<T, O>,
    completion: Completion<T>,
    started: bool,
    finished: bool,
}

impl<T, O> IosNotificationsFuture<T, O> {
    fn new(
        center: &Retained<UNUserNotificationCenter>,
        operation: O,
        start: StartOperation<T, O>,
    ) -> Self {
        Self {
            center: center.clone(),
            operation: Some(operation),
            start,
            completion: Arc::new(CompletionCell::new()),
            started: false,
            finished: false,
        }
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let Some(operation) = self.operation.take() else {
            self.completion
                .complete(Err(NotificationError::Backend(Error::new(
                    ErrorKind::Internal,
                ))));
            return;
        };
        let start = self.start;
        let center = self.center.clone();
        let completion = self.completion.clone();
        if catch_unwind(AssertUnwindSafe(|| start(center, operation, completion))).is_err() {
            self.completion
                .complete(Err(NotificationError::Backend(Error::new(
                    ErrorKind::Internal,
                ))));
        }
    }
}

impl<T: Unpin, O: Unpin> Future for IosNotificationsFuture<T, O> {
    type Output = Result<T, NotificationError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.start();
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T, O> Drop for IosNotificationsFuture<T, O> {
    fn drop(&mut self) {
        if !self.finished {
            self.completion.detach();
        }
    }
}

/// Portable notification operations backed by the app's shared UserNotifications center.
pub struct IosNotificationsBackend {
    center: Retained<UNUserNotificationCenter>,
}

impl IosNotificationsBackend {
    /// Creates a backend handle without requesting permission or scheduling notifications.
    pub fn new() -> Self {
        let center = autoreleasepool(|_| UNUserNotificationCenter::currentNotificationCenter());
        Self { center }
    }

    /// Borrows the native notification center for iOS-specific operations.
    ///
    /// Direct operations through this handle share identifiers and pending requests with the
    /// portable backend and can race with its pending-request query/removal sequence.
    pub fn native_notification_center(&self) -> &UNUserNotificationCenter {
        &self.center
    }
}

impl Default for IosNotificationsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationBackend for IosNotificationsBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    type AuthorizationFuture<'a>
        = IosAuthorizationFuture
    where
        Self: 'a;

    fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
        IosNotificationsFuture::new(&self.center, (), start_authorization)
    }

    type RequestAuthorizationFuture<'a>
        = IosRequestAuthorizationFuture
    where
        Self: 'a;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        IosNotificationsFuture::new(&self.center, (), start_request_authorization)
    }

    type ScheduleFuture<'a>
        = IosScheduleFuture
    where
        Self: 'a;

    fn schedule<'a>(&'a mut self, notification: Notification) -> Self::ScheduleFuture<'a> {
        IosNotificationsFuture::new(&self.center, notification, start_schedule)
    }

    type CancelFuture<'a>
        = IosCancelFuture
    where
        Self: 'a;

    fn cancel<'a>(&'a mut self, identifier: NotificationId) -> Self::CancelFuture<'a> {
        IosNotificationsFuture::new(&self.center, identifier, start_cancel)
    }
}

fn start_authorization(
    center: Retained<UNUserNotificationCenter>,
    (): (),
    completion: Completion<AuthorizationState>,
) {
    let handler = RcBlock::new(
        move |settings: core::ptr::NonNull<UNNotificationSettings>| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: UserNotifications supplies a non-null settings object valid for this
                    // completion-handler invocation.
                    let settings = unsafe { settings.as_ref() };
                    Ok(authorization_state_from_native(
                        settings.authorizationStatus().0,
                    ))
                })
            }))
            .unwrap_or_else(|_| Err(internal_error()));
            completion.complete(result);
        },
    );
    // UserNotifications copies and retains this completion block for its asynchronous query.
    center.getNotificationSettingsWithCompletionHandler(&handler);
}

fn start_request_authorization(
    center: Retained<UNUserNotificationCenter>,
    (): (),
    completion: Completion<AuthorizationState>,
) {
    let query_center = center.clone();
    let query_completion = completion.clone();
    let handler = RcBlock::new(move |_granted: Bool, error: *mut NSError| {
        let result = catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: NSError is borrowed for the duration of Apple's authorization callback.
                if let Some(error) = unsafe { error.as_ref() } {
                    return Err(native_error(error));
                }
                let completion = query_completion.clone();
                let settings_handler = RcBlock::new(
                    move |settings: core::ptr::NonNull<UNNotificationSettings>| {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            autoreleasepool(|_| {
                                // SAFETY: UserNotifications supplies a non-null settings object
                                // valid for this completion-handler invocation.
                                let settings = unsafe { settings.as_ref() };
                                Ok(authorization_state_from_native(
                                    settings.authorizationStatus().0,
                                ))
                            })
                        }))
                        .unwrap_or_else(|_| Err(internal_error()));
                        completion.complete(result);
                    },
                );
                // UserNotifications copies and retains this completion block for the query.
                query_center.getNotificationSettingsWithCompletionHandler(&settings_handler);
                Ok(())
            })
        }));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                query_completion.complete(Err(error));
            }
            Err(_) => {
                query_completion.complete(Err(internal_error()));
            }
        }
    });
    // The prompt-capable native call is reached only after the Rust future is first polled.
    center
        .requestAuthorizationWithOptions_completionHandler(UNAuthorizationOptions::Alert, &handler);
}

fn start_schedule(
    center: Retained<UNUserNotificationCenter>,
    notification: Notification,
    completion: Completion<()>,
) {
    let request = match catch_unwind(AssertUnwindSafe(|| make_request(notification))) {
        Ok(Ok(request)) => request,
        Ok(Err(error)) => {
            completion.complete(Err(error));
            return;
        }
        Err(_) => {
            completion.complete(Err(internal_error()));
            return;
        }
    };
    let handler = RcBlock::new(move |error: *mut NSError| {
        let result = catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: NSError is borrowed for the duration of Apple's scheduling callback.
                unsafe { error.as_ref() }.map_or(Ok(()), |error| Err(native_error(error)))
            })
        }))
        .unwrap_or_else(|_| Err(internal_error()));
        completion.complete(result);
    });
    // The center retains the copied request and completion block until it accepts or rejects it.
    center.addNotificationRequest_withCompletionHandler(&request, Some(&handler));
}

fn start_cancel(
    center: Retained<UNUserNotificationCenter>,
    identifier: NotificationId,
    completion: Completion<bool>,
) {
    let native_identifier = NSString::from_str(identifier.as_str());
    let callback_claimed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handler_center = center.clone();
    let handler_identifier = native_identifier.clone();
    let handler_completion = completion.clone();
    let handler_claimed = callback_claimed.clone();
    let handler = RcBlock::new(
        move |requests: core::ptr::NonNull<NSArray<UNNotificationRequest>>| {
            if handler_claimed.swap(true, std::sync::atomic::Ordering::AcqRel) {
                return;
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: UserNotifications supplies a non-null pending-request array that
                    // remains valid for this completion-handler invocation.
                    let requests = unsafe { requests.as_ref() };
                    let existed = requests
                        .to_vec()
                        .iter()
                        .any(|request| request.identifier().isEqualToString(&handler_identifier));
                    if existed {
                        let identifiers = NSArray::from_retained_slice(core::slice::from_ref(
                            &handler_identifier,
                        ));
                        handler_center
                            .removePendingNotificationRequestsWithIdentifiers(&identifiers);
                    }
                    Ok(existed)
                })
            }))
            .unwrap_or_else(|_| Err(internal_error()));
            handler_completion.complete(result);
        },
    );
    // UserNotifications snapshots current pending requests asynchronously for this app.
    center.getPendingNotificationRequestsWithCompletionHandler(&handler);
}

fn make_request(
    notification: Notification,
) -> Result<Retained<UNNotificationRequest>, NotificationError> {
    autoreleasepool(|_| {
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(notification.content().title()));
        if let Some(body) = notification.content().body() {
            content.setBody(&NSString::from_str(body));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| NotificationError::Backend(Error::new(ErrorKind::Unavailable)))?;
        let components =
            utc_trigger_components(notification.trigger().unix_timestamp_millis(), now)?;
        let trigger = components.map(make_calendar_trigger).transpose()?;
        let identifier = NSString::from_str(notification.identifier().as_str());
        let trigger = trigger.as_deref().map(|trigger| trigger.as_super());
        Ok(
            UNNotificationRequest::requestWithIdentifier_content_trigger(
                &identifier,
                content.as_super(),
                trigger,
            ),
        )
    })
}

fn make_calendar_trigger(
    value: UtcDateComponents,
) -> Result<Retained<UNCalendarNotificationTrigger>, NotificationError> {
    let calendar_identifier = NSString::from_str("gregorian");
    let calendar =
        NSCalendar::calendarWithIdentifier(&calendar_identifier).ok_or_else(internal_error)?;
    let time_zone = NSTimeZone::timeZoneForSecondsFromGMT(0);
    calendar.setTimeZone(&time_zone);
    let components = NSDateComponents::new();
    components.setCalendar(Some(&calendar));
    components.setTimeZone(Some(&time_zone));
    components.setYear(value.year as isize);
    components.setMonth(value.month as isize);
    components.setDay(value.day as isize);
    components.setHour(value.hour as isize);
    components.setMinute(value.minute as isize);
    components.setSecond(value.second as isize);
    components.setNanosecond(value.nanosecond as isize);
    Ok(
        UNCalendarNotificationTrigger::triggerWithDateMatchingComponents_repeats(
            &components,
            false,
        ),
    )
}

fn native_error(error: &NSError) -> NotificationError {
    let mut framework_error = Error::new(ErrorKind::Platform);
    if let Ok(code) = i32::try_from(error.code())
        && let Some(code) = PlatformErrorCode::new(code)
    {
        framework_error = framework_error.with_platform_code(code);
    }
    NotificationError::Backend(framework_error)
}

fn internal_error() -> NotificationError {
    NotificationError::Backend(Error::new(ErrorKind::Internal))
}
