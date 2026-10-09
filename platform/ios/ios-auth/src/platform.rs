use crate::operation::CompletionCell;
use block2::RcBlock;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_auth::{
    AuthenticationBackend, AuthenticationError, AuthenticationPolicy, AuthenticationRequest,
};
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use objc2::available;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{
    LAContext, LAPolicy, kLAErrorAppCancel, kLAErrorBiometryLockout, kLAErrorBiometryNotAvailable,
    kLAErrorBiometryNotEnrolled, kLAErrorDomain, kLAErrorInvalidContext, kLAErrorPasscodeNotSet,
    kLAErrorSystemCancel, kLAErrorUserCancel, kLAErrorUserFallback,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A statically selected iOS LocalAuthentication backend with no shared context
pub struct IosAuthenticationBackend {
    _not_send: PhantomData<Rc<()>>,
}

/// A lazy one-shot LocalAuthentication policy evaluation
pub struct IosAuthenticationFuture<'a> {
    _backend: &'a mut IosAuthenticationBackend,
    request: Option<AuthenticationRequest<'a>>,
    completion: Arc<CompletionCell>,
    context: Option<Retained<LAContext>>,
    reason: Option<Retained<NSString>>,
    reply: Option<RcBlock<dyn Fn(Bool, *mut NSError)>>,
    started: bool,
    finished: bool,
    _not_send: PhantomData<Rc<()>>,
}

impl IosAuthenticationBackend {
    /// Creates a backend with no process-global or native state
    pub const fn new() -> Self {
        Self {
            _not_send: PhantomData,
        }
    }
}

impl Default for IosAuthenticationBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthenticationBackend for IosAuthenticationBackend {
    fn availability(&self, policy: AuthenticationPolicy) -> Availability {
        let policy = match native_policy(policy) {
            Ok(policy) => policy,
            Err(error) if error.kind() == ErrorKind::Unsupported => {
                return Availability::Unsupported;
            }
            Err(_) => return Availability::Unknown,
        };
        // SAFETY: LAContext has a public default initializer and this owned value stays live
        // through the policy query below with no prompt
        let context = unsafe { LAContext::new() };
        // SAFETY: `policy` is one of this backend's supported policies and `context` is live
        match unsafe { context.canEvaluatePolicy_error(policy) } {
            Ok(()) => Availability::Available,
            Err(error) => availability_from_native(&error),
        }
    }

    type AuthenticateFuture<'a>
        = IosAuthenticationFuture<'a>
    where
        Self: 'a;

    fn authenticate<'a>(
        &'a mut self,
        request: AuthenticationRequest<'a>,
    ) -> Self::AuthenticateFuture<'a> {
        IosAuthenticationFuture {
            _backend: self,
            request: Some(request),
            completion: Arc::new(CompletionCell::new()),
            context: None,
            reason: None,
            reply: None,
            started: false,
            finished: false,
            _not_send: PhantomData,
        }
    }
}

impl IosAuthenticationFuture<'_> {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let (policy, reason) = match self.request.take() {
            Some(request) => {
                if request.reason().trim().is_empty() {
                    self.completion
                        .complete(Err(AuthenticationError::InvalidReason));
                    return;
                }
                let policy = match native_policy(request.policy()) {
                    Ok(policy) => policy,
                    Err(error) => {
                        self.completion.complete(Err(error));
                        return;
                    }
                };
                (policy, NSString::from_str(request.reason()))
            }
            None => {
                self.completion.complete(Err(internal_error()));
                return;
            }
        };
        // SAFETY: LAContext has a public default initializer; this future retains the context
        // until a result or drop
        let context = unsafe { LAContext::new() };
        let callback_completion = Arc::clone(&self.completion);
        let callback_claimed = Arc::new(AtomicBool::new(false));
        let reply: RcBlock<dyn Fn(Bool, *mut NSError)> =
            RcBlock::new(move |success: Bool, native_error: *mut NSError| {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    if callback_claimed.swap(true, Ordering::AcqRel) {
                        return;
                    }
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        autoreleasepool(|_| {
                            if success.as_bool() {
                                return Ok(());
                            }
                            // SAFETY: LocalAuthentication supplies a valid NSError pointer or null
                            // for the duration of this reply callback
                            let Some(error) = (unsafe { native_error.as_ref() }) else {
                                return Err(internal_error());
                            };
                            Err(error_from_native(error))
                        })
                    }))
                    .unwrap_or_else(|_| Err(internal_error()));
                    callback_completion.complete(result);
                }));
            });
        self.context = Some(context.clone());
        self.reason = Some(reason.clone());
        self.reply = Some(reply.clone());
        // SAFETY: the future owns the context and reason through callback or drop; the reply
        // captures only Arc-backed mutex/atomic state, which is Send + Sync, and no context
        unsafe { context.evaluatePolicy_localizedReason_reply(policy, &reason, &reply) };
    }

    fn finish(&mut self) {
        self.finished = true;
        self.context.take();
        self.reason.take();
        self.reply.take();
    }
}

impl Future for IosAuthenticationFuture<'_> {
    type Output = Result<(), AuthenticationError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        this.start();
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finish();
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosAuthenticationFuture<'_> {
    fn drop(&mut self) {
        self.completion.detach();
        if self.started
            && !self.finished
            && let Some(context) = self.context.as_ref()
            && available!(ios = 9.0, ..)
        {
            // SAFETY: `context` is retained by this future and iOS 9.0 adds `invalidate`
            unsafe { context.invalidate() };
        }
    }
}

fn native_policy(policy: AuthenticationPolicy) -> Result<LAPolicy, AuthenticationError> {
    match policy {
        AuthenticationPolicy::BiometricsOnly => {
            Ok(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
        }
        AuthenticationPolicy::DeviceOwner if available!(ios = 9.0, ..) => {
            Ok(LAPolicy::DeviceOwnerAuthentication)
        }
        AuthenticationPolicy::DeviceOwner => Err(unsupported_error()),
        _ => Err(unsupported_error()),
    }
}

fn availability_from_native(error: &NSError) -> Availability {
    if !is_local_authentication_error(error) {
        return Availability::Unknown;
    }
    match error.code() {
        code if code == kLAErrorPasscodeNotSet as isize
            || code == kLAErrorBiometryNotAvailable as isize
            || code == kLAErrorBiometryNotEnrolled as isize
            || code == kLAErrorBiometryLockout as isize =>
        {
            Availability::TemporarilyUnavailable
        }
        _ => Availability::Unknown,
    }
}

fn error_from_native(error: &NSError) -> AuthenticationError {
    if !is_local_authentication_error(error) {
        return backend_error(ErrorKind::Platform, None);
    }
    let code = error.code();
    let platform_code = i32::try_from(code).ok().and_then(PlatformErrorCode::new);
    let kind = match code {
        value
            if value == kLAErrorUserCancel as isize
                || value == kLAErrorUserFallback as isize
                || value == kLAErrorSystemCancel as isize
                || value == kLAErrorAppCancel as isize =>
        {
            ErrorKind::Cancelled
        }
        value if value == kLAErrorInvalidContext as isize => ErrorKind::Internal,
        _ => ErrorKind::Platform,
    };
    backend_error(kind, platform_code)
}

fn is_local_authentication_error(error: &NSError) -> bool {
    let domain = error.domain().to_string();
    kLAErrorDomain
        .to_str()
        .is_ok_and(|local_domain| domain == local_domain)
}

fn backend_error(kind: ErrorKind, code: Option<PlatformErrorCode>) -> AuthenticationError {
    let error = Error::new(kind);
    AuthenticationError::Backend(match code {
        Some(code) => error.with_platform_code(code),
        None => error,
    })
}

fn unsupported_error() -> AuthenticationError {
    AuthenticationError::Backend(Error::new(ErrorKind::Unsupported))
}

fn internal_error() -> AuthenticationError {
    AuthenticationError::Backend(Error::new(ErrorKind::Internal))
}
