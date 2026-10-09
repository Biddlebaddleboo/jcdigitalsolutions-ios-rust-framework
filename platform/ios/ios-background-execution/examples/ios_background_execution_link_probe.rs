#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_background_execution::BackgroundExecution;
#[cfg(target_os = "ios")]
use ios_background_execution::{ExpirySignal, IosBackgroundExecution};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let execution = BackgroundExecution::new(IosBackgroundExecution::new());
    if let Ok(lease) = execution.begin(main_thread) {
        let expiry = lease.expiry_signal();
        core::hint::black_box(expiry.is_expired());
        lease.end();
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
