use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use framework_background_execution::{
    BackgroundExecutionBackend, BackgroundExecutionLease, Error, ErrorKind, ExpirySignal, Result,
};
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_ui_kit::{UIApplication, UIBackgroundTaskInvalid};

use crate::lease_policy::{LeaseState, OneShot};

/// Zero-state adapter for UIKit background-execution leases.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosBackgroundExecution;

impl IosBackgroundExecution {
    /// Creates the UIKit lease adapter.
    pub const fn new() -> Self {
        Self
    }

    /// Begins a UIKit lease with a one-shot expiration callback.
    ///
    /// UIKit invokes the callback synchronously on the main thread. It runs after expiry is
    /// signaled and before the adapter ends the native identifier. Keep it bounded; a panic is
    /// contained, and explicit lease end or drop cancels a callback that has not run.
    pub fn begin_with_expiration_handler<F>(
        &self,
        main_thread: MainThread,
        expiration_handler: F,
    ) -> Result<IosBackgroundLease>
    where
        F: FnOnce() + 'static,
    {
        self.begin_inner(main_thread, Some(Box::new(expiration_handler)))
    }

    fn begin_inner(
        &self,
        main_thread: MainThread,
        expiration_handler: Option<ExpirationHandler>,
    ) -> Result<IosBackgroundLease> {
        let marker = main_thread.into_objc2();
        let application = UIApplication::sharedApplication(marker);
        let state = Arc::new(LeaseState::default());
        let state_for_expiry = Arc::clone(&state);
        let expiration_callback = expiration_handler.map(OneShot::new);
        let callback_for_expiry = expiration_callback.clone();
        let expiration_block = RcBlock::new(move || {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                state_for_expiry.handle_expiration(
                    callback_for_expiry.as_ref(),
                    || MainThreadMarker::new(),
                    |marker, identifier| {
                        let application = UIApplication::sharedApplication(marker);
                        application.endBackgroundTask(identifier);
                    },
                );
            }));
        });
        let identifier =
            application.beginBackgroundTaskWithExpirationHandler(Some(&expiration_block));
        // SAFETY: the generated binding exposes UIKit's immutable invalid-identifier constant.
        let invalid_identifier = unsafe { UIBackgroundTaskInvalid };
        if identifier == invalid_identifier {
            if let Some(callback) = &expiration_callback {
                callback.cancel();
            }
            return Err(Error::new(ErrorKind::Unavailable));
        }

        if let Some(identifier) = state.install_identifier(identifier) {
            application.endBackgroundTask(identifier);
        }
        let expiry = IosBackgroundExpiry(Arc::clone(&state));
        Ok(IosBackgroundLease {
            application,
            _main_thread: marker,
            expiry,
            expiration_callback,
        })
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
    expiration_callback: Option<OneShot<ExpirationHandler>>,
}

type ExpirationHandler = Box<dyn FnOnce() + 'static>;

impl IosBackgroundLease {
    /// Returns a cloneable signal that work can poll from another thread.
    pub fn expiry_signal(&self) -> IosBackgroundExpiry {
        self.expiry.clone()
    }

    /// Ends the native UIKit lease once and consumes this handle.
    pub fn end(self) {
        self.cancel_expiration_callback();
        self.end_native_once();
    }

    fn cancel_expiration_callback(&self) {
        if let Some(callback) = &self.expiration_callback {
            callback.cancel();
        }
    }

    fn end_native_once(&self) {
        if let Some(identifier) = self.expiry.0.end_once() {
            self.application.endBackgroundTask(identifier);
        }
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
        self.cancel_expiration_callback();
        self.end_native_once();
    }
}

impl BackgroundExecutionBackend for IosBackgroundExecution {
    type Context = MainThread;
    type Lease = IosBackgroundLease;

    fn begin(&self, main_thread: Self::Context) -> Result<Self::Lease> {
        self.begin_inner(main_thread, None)
    }
}
