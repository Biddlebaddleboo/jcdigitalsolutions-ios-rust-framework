use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use block2::RcBlock;
use framework_background_execution::{
    BackgroundExecutionBackend, BackgroundExecutionLease, Error, ErrorKind, ExpirySignal, Result,
};
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_ui_kit::{UIApplication, UIBackgroundTaskIdentifier, UIBackgroundTaskInvalid};

const ID_READY: u8 = 1;
const EXPIRED: u8 = 2;
const ENDED: u8 = 4;

/// Zero-state adapter for UIKit background-execution leases.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosBackgroundExecution;

impl IosBackgroundExecution {
    /// Creates the UIKit lease adapter.
    pub const fn new() -> Self {
        Self
    }
}

/// Thread-safe cooperative signal set by UIKit's expiration handler.
#[derive(Clone)]
pub struct IosBackgroundExpiry(Arc<LeaseState>);

impl ExpirySignal for IosBackgroundExpiry {
    fn is_expired(&self) -> bool {
        self.0.is_expired()
    }
}

/// Main-thread-bound handle for one UIKit background-task identifier.
///
/// The retained main-thread proof keeps this handle `!Send` and `!Sync`; explicit end and fallback
/// drop therefore remain on the thread that began the task.
#[must_use = "end the UIKit background-execution lease promptly"]
pub struct IosBackgroundLease {
    application: Retained<UIApplication>,
    _main_thread: MainThreadMarker,
    expiry: IosBackgroundExpiry,
}

impl IosBackgroundLease {
    /// Returns a cloneable signal that work can poll from another thread.
    pub fn expiry_signal(&self) -> IosBackgroundExpiry {
        self.expiry.clone()
    }

    /// Ends the native UIKit lease once and consumes this handle.
    pub fn end(self) {
        self.expiry.0.end_once(&self.application);
    }
}

impl BackgroundExecutionLease for IosBackgroundLease {
    type Expiry = IosBackgroundExpiry;

    fn expiry(&self) -> &Self::Expiry {
        &self.expiry
    }

    fn end(self) {
        IosBackgroundLease::end(self);
    }
}

impl Drop for IosBackgroundLease {
    fn drop(&mut self) {
        self.expiry.0.end_once(&self.application);
    }
}

impl BackgroundExecutionBackend for IosBackgroundExecution {
    type Context = MainThread;
    type Lease = IosBackgroundLease;

    fn begin(&self, main_thread: Self::Context) -> Result<Self::Lease> {
        let marker = main_thread.into_objc2();
        let application = UIApplication::sharedApplication(marker);
        let state = Arc::new(LeaseState::default());
        let state_for_expiry = Arc::clone(&state);
        let expiration_handler = RcBlock::new(move || {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                state_for_expiry.mark_expired();
                if let Some(marker) = MainThreadMarker::new() {
                    let application = UIApplication::sharedApplication(marker);
                    state_for_expiry.end_once(&application);
                }
            }));
        });
        let identifier =
            application.beginBackgroundTaskWithExpirationHandler(Some(&expiration_handler));
        // SAFETY: the generated binding exposes UIKit's immutable invalid-identifier constant.
        let invalid_identifier = unsafe { UIBackgroundTaskInvalid };
        if identifier == invalid_identifier {
            return Err(Error::new(ErrorKind::Unavailable));
        }

        state.install_identifier(identifier, &application);
        let expiry = IosBackgroundExpiry(Arc::clone(&state));
        Ok(IosBackgroundLease {
            application,
            _main_thread: marker,
            expiry,
        })
    }
}

#[derive(Default)]
struct LeaseState {
    flags: AtomicU8,
    identifier: AtomicUsize,
}

impl LeaseState {
    fn is_expired(&self) -> bool {
        self.flags.load(Ordering::Acquire) & EXPIRED != 0
    }

    fn mark_expired(&self) {
        self.flags.fetch_or(EXPIRED, Ordering::AcqRel);
    }

    fn install_identifier(
        &self,
        identifier: UIBackgroundTaskIdentifier,
        application: &UIApplication,
    ) {
        self.identifier.store(identifier, Ordering::Release);
        let previous = self.flags.fetch_or(ID_READY, Ordering::AcqRel);
        if previous & EXPIRED != 0 {
            self.end_once(application);
        }
    }

    fn end_once(&self, application: &UIApplication) {
        loop {
            let current = self.flags.load(Ordering::Acquire);
            if current & ID_READY == 0 || current & ENDED != 0 {
                return;
            }
            if self
                .flags
                .compare_exchange(
                    current,
                    current | ENDED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                let identifier = self.identifier.load(Ordering::Acquire);
                application.endBackgroundTask(identifier);
                return;
            }
        }
    }
}
