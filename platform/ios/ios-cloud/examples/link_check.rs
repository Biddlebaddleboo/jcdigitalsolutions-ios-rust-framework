#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use framework_cloud::CloudAccount;
#[cfg(target_os = "ios")]
use ios_cloud::IosCloudAccountBackend;
#[cfg(target_os = "ios")]
use std::task::{Context, Waker};

#[cfg(target_os = "ios")]
fn main() {
    let mut cloud = CloudAccount::new(IosCloudAccountBackend::new());
    let mut future = std::pin::pin!(cloud.account_status());
    let mut context = Context::from_waker(Waker::noop());
    let _ = future.as_mut().poll(&mut context);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
