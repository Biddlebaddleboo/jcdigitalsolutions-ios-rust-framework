#![deny(warnings)]

#[cfg(target_os = "ios")]
use block2::RcBlock;
#[cfg(target_os = "ios")]
use objc2_photos::{PHAccessLevel, PHAuthorizationStatus, PHPhotoLibrary};

#[cfg(target_os = "ios")]
fn main() {
    // This probe is linked for symbol/import inspection and must not be run.
    let _ = core::hint::black_box(unsafe {
        PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite)
    });
    let handler = RcBlock::new(|_status: PHAuthorizationStatus| {});
    unsafe {
        PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(
            PHAccessLevel::ReadWrite,
            &handler,
        )
    };
}

#[cfg(not(target_os = "ios"))]
fn main() {}
