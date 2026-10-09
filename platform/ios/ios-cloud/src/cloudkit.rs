use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};
use framework_cloud::{AccountStatus, AccountStatusSnapshot, CloudAccountBackend};
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use objc2::rc::{Retained, autoreleasepool};
use objc2_cloud_kit::{CKAccountStatus, CKContainer};
use objc2_foundation::NSError;
use std::{
    convert::TryFrom,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, MutexGuard},
};

/// A stateless backend for one-shot status queries on the app's default CloudKit container.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosCloudAccountBackend;

impl IosCloudAccountBackend {
    /// Creates a backend that queries the app's default CloudKit container.
    pub const fn new() -> Self {
        Self
    }
}

impl CloudAccountBackend for IosCloudAccountBackend {
    type AccountStatusFuture<'a> = IosCloudAccountStatusFuture<'a>;

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn account_status<'a>(&'a mut self) -> Self::AccountStatusFuture<'a> {
        IosCloudAccountStatusFuture::new(self)
    }
}

/// A lazy one-shot status query whose CloudKit completion may arrive on a background queue.
///
/// Dropping the future abandons Rust interest but cannot cancel the native account-status query.
/// Its callback keeps only owned scalar state until CloudKit invokes it, and does not keep any
/// CloudKit account or database data.
pub struct IosCloudAccountStatusFuture<'a> {
    _backend: &'a mut IosCloudAccountBackend,
    completion: Arc<Completion>,
    started: bool,
    finished: bool,
}

impl<'a> IosCloudAccountStatusFuture<'a> {
    fn new(backend: &'a mut IosCloudAccountBackend) -> Self {
        Self {
            _backend: backend,
            completion: Arc::new(Completion::new()),
            started: false,
            finished: false,
        }
    }
}

impl Future for IosCloudAccountStatusFuture<'_> {
    type Output = Result<AccountStatusSnapshot, Error>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        if let Poll::Ready(result) = this.completion.poll(context) {
            this.finished = true;
            return Poll::Ready(result);
        }
        if !this.started {
            this.started = true;
            start_status_query(Arc::clone(&this.completion));
        }
        if let Poll::Ready(result) = this.completion.poll(context) {
            this.finished = true;
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}

impl Drop for IosCloudAccountStatusFuture<'_> {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

struct Completion {
    state: Mutex<CompletionState>,
}

struct CompletionState {
    completed: bool,
    detached: bool,
    result: Option<Result<AccountStatusSnapshot, Error>>,
    waker: Option<Waker>,
}

impl Completion {
    fn new() -> Self {
        Self {
            state: Mutex::new(CompletionState {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    fn complete(&self, result: Result<AccountStatusSnapshot, Error>) -> bool {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return false;
            }
            state.completed = true;
            if state.detached {
                None
            } else {
                state.result = Some(result);
                state.waker.take()
            }
        };
        if let Some(waker) = waker {
            let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
        }
        true
    }

    fn poll(&self, context: &mut Context<'_>) -> Poll<Result<AccountStatusSnapshot, Error>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
        }
        if !state.completed
            && !state.detached
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, CompletionState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

fn start_status_query(completion: Arc<Completion>) {
    let handler = block2::RcBlock::new(move |status: CKAccountStatus, error: *mut NSError| {
        let result = catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: CloudKit's nullable NSError is borrowed only for this callback call.
                let error = unsafe { error.as_ref() }.map(map_native_error);
                Ok(AccountStatusSnapshot::new(map_status(status), error))
            })
        }))
        .unwrap_or_else(|_| Err(Error::new(ErrorKind::Internal)));
        completion.complete(result);
    });
    // SAFETY: CloudKit's default container owns the configured app container; its result has no
    // borrowed data. The query callback is escaping and the RcBlock owns a Send closure containing
    // only an Arc<Mutex<...>>; CloudKit retains its block copy until callback completion.
    let container: Retained<CKContainer> = unsafe { CKContainer::defaultContainer() };
    // SAFETY: `handler` is an owned heap block with a Send closure. CloudKit copies and retains
    // this asynchronous completion handler, as documented for `accountStatusWithCompletionHandler:`.
    unsafe { container.accountStatusWithCompletionHandler(&handler) };
}

fn map_status(status: CKAccountStatus) -> AccountStatus {
    match status.0 as i64 {
        0 => AccountStatus::CouldNotDetermine,
        1 => AccountStatus::Available,
        2 => AccountStatus::Restricted,
        3 => AccountStatus::NoAccount,
        4 => AccountStatus::TemporarilyUnavailable,
        raw => AccountStatus::Unknown(raw),
    }
}

fn map_native_error(error: &NSError) -> Error {
    let framework_error = Error::new(ErrorKind::Platform);
    match i32::try_from(error.code())
        .ok()
        .and_then(PlatformErrorCode::new)
    {
        Some(code) => framework_error.with_platform_code(code),
        None => framework_error,
    }
}
