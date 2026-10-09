use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::rc::Rc;
use std::sync::Arc;

use block2::RcBlock;
use framework_vpn::{
    PersonalVpnConfigurationFlags, PersonalVpnLoadError, PersonalVpnQueryError, PersonalVpnStatus,
};
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::autoreleasepool;
use objc2_foundation::NSError;
use objc2_network_extension::NEVPNManager;

use crate::completion::Completion;

struct RequestFuture<T> {
    completion: Arc<Completion<T>>,
    _main_thread: MainThread,
    _not_send_or_sync: PhantomData<Rc<()>>,
    finished: bool,
}

impl<T> RequestFuture<T> {
    fn new(completion: Arc<Completion<T>>, main_thread: MainThread) -> Self {
        Self {
            completion,
            _main_thread: main_thread,
            _not_send_or_sync: PhantomData,
            finished: false,
        }
    }
}

impl<T: Send + 'static> Future for RequestFuture<T> {
    type Output = Result<T, PersonalVpnQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        let result = this.completion.poll(context);
        if result.is_ready() {
            this.finished = true;
        }
        result
    }
}

impl<T> Drop for RequestFuture<T> {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

/// A one-shot future for a caller-app Personal VPN status snapshot.
///
/// This future is main-thread-bound and is neither `Send` nor `Sync`. Its native request starts at
/// construction. Dropping it detaches the Rust result and waker but does not cancel the Apple
/// preference-load request.
pub struct IosPersonalVpnStatusFuture(RequestFuture<PersonalVpnStatus>);

impl Future for IosPersonalVpnStatusFuture {
    type Output = Result<PersonalVpnStatus, PersonalVpnQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll(context)
    }
}

/// A one-shot future for the caller app's loaded Personal VPN configuration flags.
///
/// This future is main-thread-bound and is neither `Send` nor `Sync`. Its native request starts at
/// construction. Dropping it detaches the Rust result and waker but does not cancel the Apple
/// preference-load request.
pub struct IosPersonalVpnConfigurationFuture(RequestFuture<PersonalVpnConfigurationFlags>);

impl Future for IosPersonalVpnConfigurationFuture {
    type Output = Result<PersonalVpnConfigurationFlags, PersonalVpnQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll(context)
    }
}

pub(crate) fn start(main_thread: MainThread) -> IosPersonalVpnStatusFuture {
    IosPersonalVpnStatusFuture(start_request(main_thread, read_status))
}

pub(crate) fn start_configuration(main_thread: MainThread) -> IosPersonalVpnConfigurationFuture {
    IosPersonalVpnConfigurationFuture(start_request(main_thread, read_configuration_flags))
}

fn read_status(manager: &NEVPNManager) -> PersonalVpnStatus {
    // SAFETY: This runs only after a successful preferences load. `manager` is the retained
    // caller-process singleton and remains alive through the retained connection read.
    let connection = unsafe { manager.connection() };
    // SAFETY: `connection` is retained and live; this reads only its current status.
    let status = unsafe { connection.status() };
    PersonalVpnStatus::from_native_raw(status.0)
}

fn read_configuration_flags(manager: &NEVPNManager) -> PersonalVpnConfigurationFlags {
    // SAFETY: This runs only after a successful preferences load. `manager` is the retained
    // caller-process singleton; this reads only the public `enabled` configuration property.
    let enabled = unsafe { manager.isEnabled() };
    // SAFETY: The same retained manager is live; this reads only its `onDemandEnabled` property.
    let on_demand_enabled = unsafe { manager.isOnDemandEnabled() };
    PersonalVpnConfigurationFlags::new(enabled, on_demand_enabled)
}

fn start_request<T: Send + 'static>(
    main_thread: MainThread,
    read: fn(&NEVPNManager) -> T,
) -> RequestFuture<T> {
    let completion = Arc::new(Completion::<T>::new());
    let callback_completion = Arc::clone(&completion);
    let handler = RcBlock::new(move |native_error: *mut NSError| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                // SAFETY: Apple documents that this preference-load completion runs on the main
                // thread of the calling process. Check the documented contract before any
                // NetworkExtension query in this callback.
                let Some(_marker) = MainThreadMarker::new() else {
                    return Err(PersonalVpnQueryError::CallbackThreadMismatch);
                };

                // SAFETY: NetworkExtension passes either a null pointer on success or a live
                // NSError pointer valid for this completion call. The borrowed object is copied to
                // Rust-owned domain and code values before the callback returns.
                if let Some(error) = unsafe { native_error.as_ref() } {
                    return Err(PersonalVpnQueryError::PreferenceLoad(
                        PersonalVpnLoadError::new(error.domain().to_string(), error.code()),
                    ));
                }

                // SAFETY: `sharedManager` returns the caller process's retained singleton. The
                // selected reader invokes only documented read accessors after successful load.
                let manager = unsafe { NEVPNManager::sharedManager() };
                Ok(read(&manager))
            })
        }))
        .unwrap_or(Err(PersonalVpnQueryError::CallbackPanicked));
        callback_completion.complete(result);
    });

    // SAFETY: iOS 8.0 is the availability floor for `sharedManager` and the preference-load API.
    // The retained singleton stays alive through this call. NetworkExtension retains/copies the
    // escaping block for its asynchronous completion; the block owns an `Arc<Completion>` and
    // contains no borrowed Rust state or Objective-C handle. Apple documents main-thread callback
    // delivery, and the returned future retains `MainThread` plus a non-send marker so the caller
    // cannot safely move it off that thread.
    let manager = unsafe { NEVPNManager::sharedManager() };
    // SAFETY: `handler` is an owned heap block valid for the call, and NetworkExtension stores
    // its copy until completion. This operation only loads this app's preferences; it does not
    // modify, enable, start, or stop a VPN configuration.
    unsafe { manager.loadFromPreferencesWithCompletionHandler(&handler) };

    RequestFuture::new(completion, main_thread)
}
