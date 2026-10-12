#![deny(warnings)]

#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Poll, Waker};
#[cfg(target_os = "ios")]
use framework_connectivity::{NetworkPathBackend, NetworkPathError, NetworkPathMonitor};
#[cfg(target_os = "ios")]
use ios_connectivity::{IosConnectivityBackend, IosNetworkPathMonitorBackend};

#[cfg(target_os = "ios")]
fn main() {
    let mut backend = IosConnectivityBackend::new();
    let mut future = pin!(backend.current_path());
    let mut context = Context::from_waker(Waker::noop());
    let _: Poll<Result<_, NetworkPathError>> = future.as_mut().poll(&mut context);

    let monitor = NetworkPathMonitor::new(IosNetworkPathMonitorBackend::new());
    let _ = monitor.start();
    monitor.cancel();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
